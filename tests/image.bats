setup() {
    load '/bats/bats-support/load.bash'
    load '/bats/bats-assert/load.bash'
    cd /work/tests/image
}

@test "image: pack" {
    mkdir -p .vagga/images

    vagga _pack_image image -f .vagga/images/image.tar
    vagga _pack_image image -f .vagga/images/image.tar.gz -z
    vagga _pack_image image -f .vagga/images/image.tar.bz2 -j
    vagga _pack_image image -f .vagga/images/image.tar.xz -J
    run file .vagga/images/image.tar
    image_tar_hashsum=$(sha256sum .vagga/images/image.tar | cut -d " " -f 1)
    [[ $output = *"image.tar: POSIX tar archive (GNU)"* ]]
    run tar -tf .vagga/images/image.tar
    [[ $output = *"/var/lib/question.txt"* ]]
    run file .vagga/images/image.tar.gz
    image_targz_hashsum=$(sha256sum .vagga/images/image.tar.gz | cut -d " " -f 1)
    [[ $output = *"image.tar.gz: gzip compressed data"* ]]
    run file .vagga/images/image.tar.bz2
    [[ $output = *"image.tar.bz2: bzip2 compressed data"* ]]
    run file .vagga/images/image.tar.xz
    [[ $output = *"image.tar.xz: XZ compressed data"* ]]

    vagga _pack_image image > .vagga/images/image-stdout.tar
    vagga _pack_image image -z > .vagga/images/image-stdout.tgz
    run file .vagga/images/image-stdout.tar
    [[ $output = *"image-stdout.tar: POSIX tar archive (GNU)"* ]]
    [[ $(sha256sum .vagga/images/image-stdout.tar | cut -d " " -f 1) = $image_tar_hashsum ]]
    run file .vagga/images/image-stdout.tgz
    [[ $output = *"image-stdout.tgz: gzip compressed data"* ]]
    [[ $(sha256sum .vagga/images/image-stdout.tgz | cut -d " " -f 1) = $image_targz_hashsum ]]
}

@test "image: push & pull" {
    container_name="image"
    container_dir="$container_name.fb9b6868"
    image_file_name="${container_dir}.tar.xz"

    vagga _build nginx
    vagga _build test-pull

    rm -rf .vagga/$container_name .vagga/.roots/$container_name.*
    run vagga _build $container_name
    assert_success
    assert_output --partial "Will clean and build it locally"

    # Pack image to cache capsule's dependencies so then
    # we will be able to run test inside an isolated network environment
    run vagga pack-image
    assert_success

    run vagga --isolate-network test-push-and-pull $container_name
    assert_success

    run cat .vagga/.volumes/nginx-logs/access.log
    access_log_output=${output}
    [[ ${access_log_output} = *"PUT /upload/images/${image_file_name} HTTP/1.1\" 201"* ]]
    [[ ${access_log_output} = *"GET /images/${image_file_name} HTTP/1.1\" 200"* ]]

    [[ $(readlink .vagga/$container_name) = ".roots/${container_dir}/root" ]]

    run cat .vagga/image/var/lib/question.txt
    assert_output "To be or not to be?"
}
