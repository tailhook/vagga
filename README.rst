=====
Vagga
=====


Vagga is a fully-userspace container system inspired by Vagrant_ and Docker_,
mainly used for development system.

Mayor Features Are:

* Running programs in linux containers (not a full virtualization like Vagrant)
* Fully userspace containers, no need for elevated privileges like Docker_
* Runs containerized process as a child of current shell, no attach/detach hell
* Images are automatically rebuilt and versioned
* Vagga has tools to manage trees of processes (so you run your
    redis-python-nginx server with one command)
* Can build a docker container from vagga environment

Status: Can build and start containers using `nix` packages manager. Few
features are missing, as well as other package managers support.

* Documentation_

.. _vagrant: http://vagrantup.com
.. _docker: http://docker.io
.. _Documentation: http://vagga.readthedocs.org
