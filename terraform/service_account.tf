resource "yandex_iam_service_account" "cr-accessor" {
    name = "cr-accessor"
    description = "container registory accessor"
}

resource "yandex_resourcemanager_folder_iam_binding" "service-account-binding" {
    folder_id = yandex_iam_service_account.cr-accessor.folder_id
    role = "container-registry.images.puller"
    members = [
        "serviceAccount:${yandex_iam_service_account.cr-accessor.id}"
    ]
}
