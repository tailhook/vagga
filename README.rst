=====
Vagga
=====


Vagga is a fully-userspace container engine inspired by Vagrant_ and Docker_,
specialized for development environments.

Note version 0.2 changed format of ``vagga.yaml`` see `Release Notes`_ and
Upgrading_ for more info.

Major Features Are:

* Running programs in linux containers (not a full virtualization like Vagrant)
* Fully userspace containers, no need for elevated privileges like for Docker_
* Runs containerized process as a child of current shell, no attach/detach hell
* Images are automatically rebuilt and versioned
* Vagga has tools to manage trees of processes (so you run your
  redis-python-nginx server with one command)
* Compatibility with `Vagrant-LXC` and Docker_

More deep `feature description in docs <http://vagga.readthedocs.org/en/latest/vagga_features.html>`_

Disclaimer: This is *beta* quality software. But since it's only used for
development environments it's safe to use for most projects. Some incompatible
changes in configuration file might be introduced until release of vagga 1.0,
but it will never affect your production servers.

Documentation_

.. _vagrant: http://vagrantup.com
.. _docker: http://docker.io
.. _Documentation: http://vagga.readthedocs.org
.. _Vagrant-LXC: https://github.com/fgrehm/vagrant-lxc
.. _Release Notes: http://github.com/tailhook/vagga/blob/master/RELEASE_NOTES.rst
.. _Upgrading: http://vagga.readthedocs.org/en/latest/upgrading.html


.. image:: https://badges.gitter.im/Join%20Chat.svg
   :alt: Join the chat at https://gitter.im/tailhook/vagga
   :target: https://gitter.im/tailhook/vagga?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge