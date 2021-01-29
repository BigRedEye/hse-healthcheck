resource "yandex_compute_instance" "nat" {
  name = "nat"

  resources {
    cores  = 2
    memory = 1
    core_fraction = 5
  }

  boot_disk {
    initialize_params {
      image_id = "fd85tqltvlg3mtufp0il"
    }
  }

  network_interface {
    subnet_id = yandex_vpc_subnet.public-subnet.id
    nat       = true
  }

  metadata = {
    ssh-keys = "ubuntu:${file("/home/sergey/.ssh/id_yandex_cloud.pub")}"
  }
}
