resource "yandex_compute_instance" "nginx" {
  name = "nginx"

  resources {
    cores  = 2
    memory = 1
    core_fraction = 5
  }

  boot_disk {
    initialize_params {
      image_id = data.yandex_compute_image.container-optimized-image.id
    }
  }

  service_account_id = yandex_iam_service_account.cr-accessor.id

  network_interface {
    subnet_id = yandex_vpc_subnet.public-subnet.id
    nat       = true
  }

  metadata = {
    ssh-keys = "ubuntu:${file("/home/sergey/.ssh/id_yandex_cloud.pub")}"
    docker-container-declaration = file("${path.module}/docker/nginx.generated.yaml")
  }
}
