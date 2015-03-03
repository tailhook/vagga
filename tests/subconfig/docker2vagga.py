# This is just an example it's not production-ready Dockerfile compatibility
# layer
from __future__ import print_function
import json

with open('Dockerfile') as f:
    text = f.read()

text = text.replace('\\\n', '')

def cmdargfilter(val):
    return not val.startswith('-')

print("containers:")
print("  docker-raw:")
print("    setup:")
for line in text.splitlines():
    if line.strip().startswith('#'):
        continue
    if line.startswith('FROM '):
        image = line.split()[1]
        assert image.startswith('ubuntu:'), image
        print("    - !Ubuntu", image[7:])
        print("    - !UbuntuUniverse") # enabled in docker by default
    elif line.startswith('RUN '):
        print("    - !Sh", repr(line[3:].strip()))

print("  docker-smart:")
print("    setup:")
for line in text.splitlines():
    if line.strip().startswith('#'):
        continue
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



