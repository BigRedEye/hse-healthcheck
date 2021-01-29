resource "yandex_vpc_network" "main-network" {
  name = "main-network"
}

resource "yandex_vpc_subnet" "private-subnet" {
  zone           = "ru-central1-a"
  network_id     = yandex_vpc_network.main-network.id
  v4_cidr_blocks = ["192.168.10.0/24"]
  route_table_id = yandex_vpc_route_table.nat-route-table.id
}

resource "yandex_vpc_subnet" "public-subnet" {
  zone           = "ru-central1-a"
  network_id     = yandex_vpc_network.main-network.id
  v4_cidr_blocks = ["10.0.0.0/24"]
}

resource "yandex_vpc_route_table" "nat-route-table" {
  network_id     = yandex_vpc_network.main-network.id

  static_route {
    destination_prefix = "0.0.0.0/0"
    next_hop_address   = yandex_compute_instance.nat.network_interface.0.ip_address
  }
}
