setup() {
    cd /work/tests/image
    mkdir -p images
    mkdir -p nginx/logs
}

@test "image: pack" {
    rm -rf images/image.*

    vagga _pack_image alpine -f images/image.tar
    vagga _pack_image alpine -f images/image.tar.gz -z
    vagga _pack_image alpine -f images/image.tar.bz2 -j
    vagga _pack_image alpine -f images/image.tar.xz -J
    run file images/image.tar
    printf "%s\n" "${lines[@]}"
    [[ $output = *"image.tar: POSIX tar archive (GNU)"* ]]
    run file images/image.tar.gz
    printf "%s\n" "${lines[@]}"
    [[ $output = *"image.tar.gz: gzip compressed data"* ]]
    run file images/image.tar.bz2
    printf "%s\n" "${lines[@]}"
    [[ $output = *"image.tar.bz2: bzip2 compressed data"* ]]
    run file images/image.tar.xz
    printf "%s\n" "${lines[@]}"
    [[ $output = *"image.tar.xz: XZ compressed data"* ]]

    vagga _pack_image alpine > images/image.tar
    vagga _pack_image alpine -z > images/image.tgz
    run file images/image.tar
    printf "%s\n" "${lines[@]}"
    [[ $output = *"image.tar: POSIX tar archive (GNU)"* ]]
    run file images/image.tgz
    printf "%s\n" "${lines[@]}"
    [[ $output = *"image.tgz: gzip compressed data"* ]]
}

@test "image: push & pull" {
    hash="eaeba474"
    container_dir="alpine.${hash}"
    image_name="${container_dir}.tar.xz"

    rm -rf /work/tmp/cache/downloads/*-${image_name}

    run vagga _build alpine
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/alpine)
    [[ $link = ".roots/${container_dir}/root" ]]

    rm -rf nginx/nginx.pid
    rm -rf nginx/logs/*
    vagga _build nginx
    vagga nginx > /dev/null 2>&1 &
    nginx_pid=$!
    sleep 2

    run vagga _push_image alpine
    printf "%s\n" "${lines[@]}"

    run tail -n 1 nginx/logs/access.log
    printf "%s\n" "${lines[@]}"
    access_log_output=${output}

    # test download
    rm -rf .vagga/alpine
    rm -rf .vagga/.roots/${container_dir}
    run vagga _build alpine
    printf "%s\n" "${lines[@]}"
    build_status=${status}
    build_link=$(readlink .vagga/alpine)

    run vagga _run alpine sh -c "echo '100*24' | bc"
    printf "%s\n" "${lines[@]}"
    run_status=${status}
    run_lines=${lines}

    kill -TERM "${nginx_pid}"

    [[ ${access_log_output} = *"PUT /upload/images/${image_name} HTTP/1.1\" 201"* ]]

    [[ ${build_status} = 0 ]]
    [[ ${build_link} = ".roots/${container_dir}/root" ]]
    [[ ${run_status} -eq 0 ]]
    [[ ${run_lines[${#run_lines[@]}-1]} = "2400" ]]

    run tail -n 1 nginx/logs/access.log
    printf "%s\n" "${lines[@]}"
    [[ ${output} = *"GET /images/${image_name} HTTP/1.1\" 200"* ]]
}
