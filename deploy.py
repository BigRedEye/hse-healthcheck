#!/usr/bin/env python3

import os
import sys
from typing import *
from jinja2 import Template
from pathlib import Path
import logging as log
import subprocess as sp

ROOT = Path(os.path.dirname(os.path.realpath(__file__)))

class CalledProcessError(Exception):
    def __init__(self, code: int, stderr: str):
        self.code = code
        self.stderr = stderr
        self.message = f'Called process failed with code: {code}, stderr: {stderr}'
        super().__init__(self.message)


def run(args: list[str], cwd: Optional[str]=None, interactive: bool=False) -> str:
    log.info(f'Running command "{args}"')
    kwargs = {'stderr': sys.stderr, 'stdout': sys.stdout} if interactive else {'capture_output': True}
    res = sp.run(args, shell=True, cwd=cwd, encoding='utf-8', **kwargs)
    if res.returncode != 0:
        log.error('Called process failed with code %d and stderr %s', res.returncode, res.stderr)
        raise CalledProcessError(res.returncode, res.stderr)
    log.info('Called process finished with code 0 and stderr %s', res.stderr)
    return res.stdout


class Env:
    ARGS = [
        ('cloud', 'YC_CLOUD_ID', 'Id of the cloud', str),
        ('folder', 'YC_FOLDER_ID', 'Id of the folder', str),
        ('token', 'YC_TOKEN', 'OAUTH token', str),
        ('cr', 'YC_CR', 'Container registry name', str),
        ('replication', 'NUM_BACKENDS', 'Number of healthcheck instances', int),
        ('dryrun', 'DRY_RUN', 'Do not apply changes', bool),
    ]

    def __init__(self):
        self.env = os.environ
        self.args = {}
        for id, key, help, type in Env.ARGS:
            value = type(self._get_param(key, help))
            setattr(self, id, value)
            self.args[key] = value

        self._patch_environ()

    def _get_param(self, key: str, help: str) -> str:
        if key in self.env:
            return self.env[key]
        else:
            return input(f'Enter value for ${key} ({help}): ')

    def _patch_environ(self) -> None:
        for key, value in self.args.items():
            self.env[key] = str(value)
        self.env['TF_VAR_replication'] = str(self.replication)


class DockerImage:
    def __init__(self, image_id: str):
        self._id = image_id
        self._tag = None

    def push(self) -> None:
        assert self.is_tagged()
        log.info(f'Pushing image {self._tag}')
        run(f'docker push {self._tag}')

    def tag(self, tag: str):
        self._tag = tag
        log.info(f'Tagging image {self._id} with tag {self._tag}')
        run(f'docker tag {self._id} {self._tag}')

    def is_tagged(self) -> bool:
        return bool(self._tag)

    def get_tag(self) -> str:
        assert self.is_tagged()
        return self._tag


class Docker:
    def build(self, cwd: str='.') -> DockerImage:
        log.info('Building docker image from directory %s' % os.path.realpath(cwd))
        image = run(f'docker build -q {cwd}').strip()
        log.info('Done building docker image %s' % image)
        return DockerImage(image)


class Service:
    def __init__(self, env: Env):
        self._env = env

    def build(self) -> None:
        self._generate_configs()
        image = self._push_image()
        self._generate_docker_declaration(image)

    def _push_image(self) -> DockerImage:
        image = self._make_image()
        image.tag(self._make_docker_tag())
        if not self._env.dryrun:
            image.push()
        else:
            log.info('Skipped image uploading')
        return image

    def _make_docker_tag(self) -> str:
        return f'cr.yandex/{self._env.cr}/{self.name()}:latest'

    def _generate_docker_declaration(self, image: DockerImage) -> None:
        ctx = {
            'image': image.get_tag(),
            'env': self._make_env(),
        }
        self._generate_config(
            src=ROOT / 'terraform' / 'docker' / 'template.jinja2',
            dst=ROOT / 'terraform' / 'docker' / (self.name() + '.generated.yaml'),
            ctx=ctx
        )

    def _generate_config(self, src: Path, dst: Path, ctx: dict) -> None:
        tmpl = self._load_jinja_template(src)
        config = tmpl.render(ctx)
        with open(dst, 'w') as f:
            f.write(config)

    def _load_jinja_template(self, path: os.PathLike) -> Template:
        return Template(open(path).read())

    def _generate_configs(self) -> None:
        pass

    def _make_image(self) -> DockerImage:
        pass

    def name(self) -> str:
        pass

    def _make_env(self) -> dict[str, str]:
        return {}


class NginxService(Service):
    def name(self) -> str:
        return 'nginx'

    def _generate_configs(self) -> None:
        ctx = {
            'backends': [f'healthcheck-{i}' for i in range(self._env.replication)],
        }
        self._generate_config(
            src=ROOT / 'nginx' / 'nginx.conf.jinja2',
            dst=ROOT / 'nginx' / 'nginx.conf.jinja2.generated',
            ctx=ctx)

    def _make_image(self) -> DockerImage:
        return Docker().build(cwd='nginx')


class PostgresService(Service):
    def name(self) -> str:
        return 'postgres'

    def _make_image(self) -> DockerImage:
        return DockerImage('postgres:13')

    def _make_env(self) -> dict[str, str]:
        return {
            'POSTGRES_PASSWORD': 'aefac2e2d9fccd1',
            'POSTGRES_USER': 'healthcheck',
        }


class HealthcheckService(Service):
    def name(self) -> str:
        return 'healthcheck'

    def _make_image(self) -> DockerImage:
        return Docker().build(cwd='app')

    def _make_env(self) -> dict[str, str]:
        return {
            'NODE_DATABASE_URL': 'postgres://healthcheck:aefac2e2d9fccd1@postgres/postgres',
            'NODE_BIND_ADDRESS': '0.0.0.0:80',
        }


def load_env() -> Env:
    return Env()


def main() -> None:
    log.basicConfig(level=log.INFO, format='%(asctime)s %(levelname)s %(message)s')
    env = load_env()

    services = [
        NginxService(env),
        PostgresService(env),
        HealthcheckService(env)
    ]
    for service in services:
        log.info('Building service %s' % service.name())
        service.build()

    log.info('Running terraform plan')
    print(run('terraform plan', cwd=ROOT / 'terraform'))

    if not env.dryrun:
        log.info('Running terraform apply')
        print(run('terraform apply -auto-approve', cwd=ROOT / 'terraform', interactive=True))
    else:
        log.info('Skipped terraform apply')

if __name__ == '__main__':
    main()
