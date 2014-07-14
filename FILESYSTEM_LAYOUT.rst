=======================
Vagga Filesystem Layout
=======================

Everthing is in `.vagga` directory. We don't pollute anything else. But
consider the following:

* Project dir is mounted as ``/work`` in the container
* The ``$HOME`` may be optionally mounted inside container


Containers
==========

* ``.vagga/xxx`` -- symlink to a container root for the container xxx
  if container has variants, it points into ``xxx-yy-zz`` symlink which is
  linked to an active (or last used variant)
* ``.vagga/xxx-yy-zz`` -- symlink to container root for the container ``xxx``
  with variant variables set to values ``yy`` and ``zz`` (in the order of
  names of the variables)
* ``.vagga/.roots/xxx.01234abc`` -- a root filesystem for the container ``xxx``
  having ``01234abc`` hash of it's version (what means by "version" depends by
  a backend). This is the target of ``.vagga/xxx`` links.
* ``.vagga/.artifacts/xxx.01234abc`` -- artifacts needed to build container
  (generated files, configs, version files, whatever.. depends on backend)
* ``.vagga/.cache/bbb`` -- the container-agnostic cache files
  for the backend ``bbb``, may be reused between containers


Configuration
=============

* ``.vagga/vagga.yaml`` -- configuration file, overrides ``vagga.yaml``, may
  also serve as config if you don't want to commit ``vagga.yaml`` into source
  control
* ``.vagga/settings.yaml`` -- settings that override something locally and are
  manipulated by vagga itself


