#!/bin/sh -xe
: ${project_root:=.}
: ${vagga_exe:=vagga}
: ${vagga_inventory:=/usr/lib/vagga/inventory}
: ${container_hash:=tmpbuildhash}
: ${container_name:=work}
: ${container_fullname:=$container_name}
: ${artifacts_dir:=$project_root/.vagga/.artifacts/$container_fullname.$container_hash}
: ${container_root:=$project_root/.vagga/.roots/$container_fullname.$container_hash}
: ${cache_dir:=$project_root/.vagga/.cache/from_image}
: ${docker_image:=}
: ${docker_dockerfile:=}

type curl
type mkdir
type awk
type sed
type tar
type mktemp


if [ -n "$docker_dockerfile" ]; then
    tmpname=$(mktemp)
    grep -vE '^\s*#' "$docker_dockerfile" | \
        sed ':x; /\\$/ { N; s/\\\n//; tx }' > $tmpname
    exec < $tmpname
    rm $tmpname
    while read -r kw tail; do
        case "$kw" in
            \#*|"") ;;
            FROM)
                if [ -z "$docker_image" ]; then
                    docker_image="$tail"
                else
                    echo "Using $docker_image instead $tail" >&2
                fi
                break
                ;;
            *)
                echo "FROM instruction not found" >&2
                exit 1
                ;;
        esac
    done
fi

if [ -z "$docker_image" ]; then
    echo 'Either `image` or `dockerfile` must be specified' >&2
    exit 1
fi


if [ "${docker_image#*/}" = "$docker_image" ]; then
    repo="library/$docker_image";
else
    repo="$docker_image"
fi
if [ "${repo%:*}" = "$repo" ]; then
    tag=latest
else
    repo="${repo%:*}"
    tag="${docker_image#*:}"
fi

mkdir -p $artifacts_dir
mkdir -p $cache_dir

curl --header "X-Docker-Token: true" --output $artifacts_dir/tags.json \
    --dump-header $artifacts_dir/tags_header.txt --insecure --location\
    https://index.docker.io/v1/repositories/$repo/tags

curl --header "X-Docker-Token: true" --output $artifacts_dir/images.json \
    --dump-header $artifacts_dir/images_header.txt --insecure --location\
    https://index.docker.io/v1/repositories/$repo/images

layer=$(${vagga_exe} _extract_json $artifacts_dir/tags.json name layer \
        | awk '/^'"$tag"'\t/{ print $2; exit 0; }')

[ -n "$layer" ] || { echo "Tag $tag not found" 2>&1; exit 1; }

image=$(${vagga_exe} _extract_json $artifacts_dir/images.json id \
        | grep --max-count=1 "^$layer")

token=$(awk 'BEGIN { RS="\r\n"; } tolower($0) ~ /^x-docker-token:/{ print $2; exit 0; } ENDFILE { exit 1; }' \
    $artifacts_dir/images_header.txt)
endpoint=$(awk 'tolower($0) ~ /^x-docker-endpoints:/{
        match($2, /[a-z0-9.-]+/, s);
        print s[0]; exit 0;}
    ENDFILE { exit 1; }' \
    $artifacts_dir/images_header.txt)

#TODO(tailhook) fix the --insecure
curl --header "Authorization: Token $token" \
    --output $artifacts_dir/image.json \
    --insecure --location \
    "https://$endpoint/v1/images/$image/json"

curl --header "Authorization: Token $token" \
    --output $artifacts_dir/ancestry.json \
    --insecure --location \
    "https://$endpoint/v1/images/$image/ancestry"

filenames=""
for image in $(${vagga_exe} _extract_json $artifacts_dir/ancestry.json ""); do
    fn="$cache_dir/layer-${image}.tar"
    filenames="$fn $filenames"
    [ -e "$fn" ] && continue
    curl \
        --header "Authorization: Token $token" \
        --output "${fn}.tmp" \
        --insecure --location \
        "https://$endpoint/v1/images/$image/layer"
    mv "${fn}.tmp" "$fn"
done


for fn in $filenames; do
    tar -xf "$fn" --no-same-owner --exclude "dev/*" -C "$container_root"
    find "$container_root" -name ".wh.*" | sed 's/\.wh\.//' \
        | xargs --no-run-if-empty rm -rf
    find "$container_root" -name ".wh.*" | xargs --no-run-if-empty rm -rf
done

[ -z "$docker_dockerfile" ] && exit 0

while read -r kw tail; do
    case "$kw" in
        \#*|"") ;;
        FROM)
            echo "FROM instruction starts new container. Stopping." >&2
            exit 0
            ;;
        RUN)
            ${vagga_exe} _chroot --writeable --inventory \
                --environ=PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
                --environ=LD_PRELOAD=/tmp/inventory/libfake.so \
                --environ=HOME=/ \
                "$container_root" \
                /bin/sh -c "$tail"
            ;;
        *)
            echo "Unknown instruction ${kw}. Ignoring." >&2
            ;;
    esac
done
