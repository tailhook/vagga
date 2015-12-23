# This is just an example it's not production-ready Dockerfile compatibility
# layer
from __future__ import print_function
import json
import sys


def cmdargfilter(val):
    return not val.startswith('-')


def main(argv):
    docker_filename = 'Dockerfile'
    if len(argv) == 2:
        docker_filename = argv[1]

    with open(docker_filename) as f:
        rows = f.readlines()

    # cut commented lines before processing continued lines
    rows = list(filter(lambda line: not line.startswith('#'), rows))

    # join continued lines
    preprocessed_rows = ''.join(rows).replace('\\\n', '').splitlines()

    print("containers:")
    print("  docker-raw:")
    print("    setup:")
    for line in preprocessed_rows:
        if line.startswith('FROM '):
            image = line.split()[1]
            assert image.startswith('ubuntu:'), image
            print("    - !Ubuntu", image[7:])
            print("    - !UbuntuUniverse") # enabled in docker by default
        elif line.startswith('RUN '):
            print("    - !Sh", repr(line[3:].strip()))

    print("  docker-smart:")
    print("    setup:")
    for line in preprocessed_rows:
        if line.startswith('FROM '):
            image = line.split()[1]
            assert image.startswith('ubuntu:'), image
            print("    - !Ubuntu", image[7:])
            print("    - !UbuntuUniverse") # enabled in docker by default
        elif line.startswith('RUN '):
            cmd = line[3:].split()
            if cmd[0] == 'apt-get':
                if cmd[1] == 'update':
                    continue
                elif cmd[1] == 'install':
                    packages = list(filter(cmdargfilter, cmd[2:]))
                    print("    - !Install", json.dumps(packages))
            elif cmd[0] == 'pip' and cmd[1] == 'install':
                packages = list(filter(cmdargfilter, cmd[2:]))
                print("    - !Py2Install", json.dumps(packages))
            else:
                print("    - !Sh", repr(line[3:].strip()))


if __name__ == '__main__':
    main(sys.argv)
