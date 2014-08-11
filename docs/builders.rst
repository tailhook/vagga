.. _builders:

==================
Container Builders
==================


Nix
===

Nix_ is currently the most featureful builder. Good support of nix is good
because it matches vagga concepts most closely. Note, that you don't need
to have nixos_ installed to use *nix* builder. You only need to have nix
package manager installed


Depenendencies
--------------

* ``nix`` package manager (commands ``nix-env``, ``nix-store``,
  ``nix-instantiate``)
* ``rsync``


Paramaters
----------

``config``
    (default: ``default.nix``) The ``.nix`` file to read configuration from.
    The path is relative to project root.

``attribute``
    (default: empty). The attribute to use for ``nix-instantiate -A``


Best Practices
--------------

Few tips:

* You almost certainly want to import ``nixpkgs``
* Use ``nixpkgs.buildEnv`` to build environment, so you can run use short
  paths for commands
* Low-level things like ``coreutils`` must be explicitly specified

For example here is how vagga documentation used to build:

.. code-block:: nix

    let
      pkgs = import <nixpkgs> { };
    in {
      sphinx = pkgs.buildEnv {
        name = "vagga-sphinx-env";
        paths = with pkgs; with pkgs.pythonPackages; [
          gnumake
          bash
          coreutils
          sphinx
        ];
      };
    }

And how it's used in ``vagga.yaml``:

.. code-block:: yaml

    containers:

      sphinx:
        builder: nix
        parameters:
          config: default.nix
          attribute: sphinx

.. _nix: https://nixos.org/nix/
.. _nixos: http://nixos.org


Archlinux
=========

Current archlinux_ builder can only setup packages from archlinux binary
repositories. In future we are going to implement building source packages with
makepkg in the container.

.. note:: There is no versioning support for this backend. This means that
   containers will be versioned barely by list of packages. This should be
   ok for most uses, but it means that builds on different machines and/or in
   different periods will vary.


Dependencies
------------

* ``pacman``
* ``wget``


Parameters
----------

``packages``
    (default: ``base``) A space-separated list of packages to install. Members
    of this list might also be package groups or requirement specifications
    (e.g. ``shadow>=4.1``) that are supported by pacman on a command-line.

``pacman_conf``
    (defaults to vagga's builtin config) A path to customized ``pacman.conf``.
    The path is relative to project root.


Tips
----

Nothing is installed by default. So usually you need ``bash`` and ``coreutils``

For example here is how container for vagga docs might be built:

.. code-block:: yaml

  sphinx-arch:
    builder: arch
    parameters:
      packages: python-sphinx make coreutils bash

.. _archlinux: http://archlinux.org


Debian-simple
=============

The ``debian_simple`` backend can be used to setup debian (or ubuntu or
probably any other debian derivative) by just unpacking ``deb`` files. No
``configure`` and ``install`` phases are run.

.. warning:: Given the complexity of debian packages and bad design of
   debootstrap we have not found a good way to install debian packages in a
   container (without root privileges). But also unlike in arch, many debian
   packages do some crazy things after unpacking, so many packages after
   unpacking do not work at all or have files located in unusual places.


Simple debian system setup:

.. code-block:: yaml

   sphinx:
     builder: debian_simple
     parameters:
       packages: python-sphinx,make

Simple ubuntu system setup:

.. code-block:: yaml

   builder: debian_simple
   parameters:
     repo: http://archive.ubuntu.com/ubuntu
     suite: trusty
     packages: python-sphinx,make


Dependencies
------------

* ``debootstrap`` (and all of its depedencies)


Parameters
----------

``repo``
    Repository for the packages. ``http://http.debian.net/debian/`` for Debian
    and ``http://archive.ubuntu.com/ubuntu`` for ubuntu.

``suite``
    The suite to run for debian it may be a version of OS or some special value
    like ``sid`` or ``stable``. Refer to debootstrap documentation for more
    info.


``arch``
    Target architecture (default should work)

``packages``
    A comma-separated packages to install


Debian Debootstrap
==================

The ``debian_debootstrap`` backend set's up debian or debian-derivative system
using ``debootstrap`` script. Unlike ``debian_simple`` backend this one runs
all debian hooks. However they may not work because of quirks we do to run
debootstrap in user namespaces.


Simple debian system setup:

.. code-block:: yaml

   sphinx:
     builder: debian_debootstrap
     parameters:
       packages: python-sphinx,make

Simple ubuntu system setup:

.. code-block:: yaml

   builder: debian_debootstrap
   parameters:
     repo: http://archive.ubuntu.com/ubuntu
     suite: trusty
     packages: python-sphinx,make

Dependencies
------------

* ``debootstrap`` (and all of its depedencies)


Parameters
----------

``repo``
    Repository for the packages. ``http://http.debian.net/debian/`` for Debian
    and ``http://archive.ubuntu.com/ubuntu`` for ubuntu.

``suite``
    The suite to run for debian it may be a version of OS or some special value
    like ``sid`` or ``stable``. Refer to debootstrap documentation for more
    info.


``arch``
    Target architecture (default should work)

``packages``
    A comma-separated packages to install


From Image
==========

The ``from_image`` backend downloads image, unpacks it, and uses that as an
image for the system. Using :ref:`Provision<provision>` you can install
additional packages or do whatever you need to configure system.

Example Ubuntu image:

.. code-block:: yaml

    builder: from_image
    parameters:
      url: http://cdimage.ubuntu.com/ubuntu-core/trusty/daily/current/trusty-core-amd64.tar.gz

Besides official ubuntu image or any other tar containing root file system
you can use official lxc_ system images: http://images.linuxcontainers.org/.
Any image listed there should work, but you must choose correct architecture
and an ``rootfs.tar.*`` file. For example this one is for ubuntu:

.. code-block:: yaml

    builder: from_image
    parameters:
      url: http://images.linuxcontainers.org/images/debian/sid/amd64/default/20140803_22:42/rootfs.tar.xz

.. _lxc: linuxcontainers.org

Dependencies
------------

* ``wget``
* ``tar``


Parameters
----------

``url``
    A url of an image.


Tips
----

When using ubuntu/debian system, you can't install packages with ``dpkg``
or ``apt-get``, because they don't like user namespaces having only few users
(we often have only root in the namespace). In this case you may use vagga's
variant of fakeroot, to avoid the problem:

.. code-block:: yaml

    builder: from_image
    parameters:
      url: http://cdimage.ubuntu.com/ubuntu-core/trusty/daily/current/trusty-core-amd64.tar.gz
    provision:
      export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin;
      export LD_PRELOAD=/tmp/inventory/libfake.so;
      apt-get -y install python3


Vagrant LXC
===========

This backend is very similar to ``from_image`` but allows to use any
vagrant-lxc_ image from `Vagrant Cloud`_ a base image for vagga container.

.. note:: it doesn't use metadata from vagrant image, only root file system
   is used

Here is an example of ubuntu container:

.. code-block:: yaml

    builder: vagrant_lxc
    parameters:
      name: fgrehm/trusty64-lxc

.. note:: same precautions that are described for ``from_image`` builder apply
   here


Dependencies
------------

* ``wget``
* ``tar``


Parameters
----------

``name``
    Name of an image on `Vagrant Cloud`_ . Should be in form
    ``username/imagename``.

``url``
    The full url for the image. Useful for images that are not on
    Vagrant Cloud. If both ``name`` and ``url`` are specified, the ``url``
    is used.

.. _vagrant-lxc: https://github.com/fgrehm/vagrant-lxc
.. _`Vagrant Cloud`: https://vagrantcloud.com/

.. _docker-builder:

Docker
======

This backend can fetch Docker_ images from a repository and/or use Dockerfiles
to build containers.

Raw ubuntu container:

.. code-block:: yaml

   ubuntu:
     builder: docker
     parameters:
       image: ubuntu

Container with dockerfile:

.. code-block:: yaml

   mycontainer:
     builder: docker
     parameters:
        dockerfile: Dockerfile


Dependencies
------------

* ``curl``
* ``awk`` (tested on gawk, other variants may work too)

.. note:: you *don't need* to have docker installed when using the builder


Parameters
----------

``image``
    Base docker image to use. Currently we only support downloading images from
    ``index.docker.io``, support of private repositories will be added later.

``dockerfile``
    Filename of the Dockerfile_ to use, relative to the project directory (the
    directory where ``vagga.yaml`` is).

.. note:: if both ``image`` and ``dockerfile`` are specified, the ``image``
   parameter overrides the one used in ``FROM``. For example you can make
   container which is built from ``ubuntu-debootstrap`` instead of
   ``FROM ubuntu``, effectively making container smaller (in some cases).


Limitations
-----------

* Only single ``FROM`` instruction supported
* Only ``RUN`` instructions are supported so far, other will be implemented
  later
* Instructions which influence command run in container will probably never
  be implemented, including ONBUILD, CMD, WORKDIR... There is :ref:`vagga
  syntax for those things<Containers>`.


.. _docker: http://docker.com
.. _Dockerfile: http://docs.docker.com/reference/builder/


Ubuntu
======

We do not have any official ubuntu builder yet. This is because
``debootstrap``, ``dpkg`` and ``apt-get`` need to have quite many quirks for
working in user namespaces (BTW, docker have plenty of hacks to get it working
too, but they are different from what we need). We are working to provide the
official best of all worlds ubuntu (and debian) container builder. In the
meantime you can use any of `Debian Debootstrap`_, `From Image`_,  `Vagrant
LXC`_, :ref:`docker-builder` or `Debian-simple`_ builders. Every of it's
section have an example on how to setup Ubuntu specifically. Please report any
issues you have with any of them.


