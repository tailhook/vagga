=================================
Welcome to Vagga's documentation!
=================================

Vagga is a tool to create development environments. In particular it is
able to:

* Build container and run program with single command, right after ``git pull``
* Automatically rebuild container if project dependencies change
* Run multiple processes (e.g. application and database) with single command
* Execute network tolerance tests

All this seamlessly works using linux namespaces (or containers).

.. hint:: Vagga is not a tool for running production. But vagga is perfect for
   building containers which with then **run in production**. It means you get
   container built by vagga transfer it to production system and start it by
   docker_, lxc_, lxd_, runc_, systemd-nspawn_, lithos_ or even chroot_.  You
   ain't need a build tool in production, right?

.. _docker: http://docker.com
.. _lxc: https://linuxcontainers.org/
.. _lxd: https://linuxcontainers.org/
.. _runc: http://runc.io
.. _systemd-nspawn: http://www.freedesktop.org/software/systemd/man/systemd-nspawn.html
.. _lithos: http://lithos.readthedocs.org
.. _chroot: http://linux.die.net/man/1/chroot

Links
=====

* `Managing Dependencies with Vagga <https://medium.com/@paulcolomiets/managing-dependencies-with-vagga-79181046db66>`_
  shows basic concepts of using vagga and what problems it solves

* `The Higher Level Package Manager <https://medium.com/@paulcolomiets/vagga-the-higher-level-package-manager-e49e85fed42a>`_ -- discussion of vagga goals and future


* `Evaluating Mesos <https://medium.com/@paulcolomiets/evaluating-mesos-4a08f85473fb>`_
  discuss how to run network tolerance tests

* `Container-only Linux Distribution <https://medium.com/p/container-only-linux-distribution-ff0497933c33>`_

Documentation Contents
======================

.. toctree::
   :maxdepth: 2

   info
   installation
   config
   running
   network
   tips
   conventions
   examples


Indices and tables
==================

* :ref:`genindex`

