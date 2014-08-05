=====
Vagga
=====


Vagga is a fully-userspace container engine inspired by Vagrant_ and Docker_,
mainly used for development environments.

Mayor Features Are:

* Running programs in linux containers (not a full virtualization like Vagrant)
* Fully userspace containers, no need for elevated privileges like for Docker_
* Runs containerized process as a child of current shell, no attach/detach hell
* Images are automatically rebuilt and versioned
* Vagga has tools to manage trees of processes (so you run your
    redis-python-nginx server with one command)
* Partial compatibility with `Vagrant-LXC` and Docker_ (pretty limited so far)

More deep `feature description at documentation <http://vagga.readthedocs.orl/vagga_features.html>`

Status: beta

* Documentation_

.. _vagrant: http://vagrantup.com
.. _docker: http://docker.io
.. _Documentation: http://vagga.readthedocs.org
.. _Vagrant-LXC: https://github.com/fgrehm/vagrant-lxc
