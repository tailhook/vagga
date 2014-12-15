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

The ``arch`` builder features:

* starts from official image (no need for ``pacman`` on host system)
* supports additional repositories
* builds additional packages with makepkg
* rebuilds container on parameter change and when any of the PKGBUILDs change

Dependencies
------------

* ``wget`` or ``curl``
* ``tar``
* ``sed``, ``awk``, ``grep``


Parameters
----------

``mirror``
    A URL to the archlinux you want to use. Used both for initial image and
    for pacman packages later

``arch``
    Architecture to install. Only ``x86_64`` tested, and that is default

``image_release``
    The date of the release to use. Default is latest one.

``packages``
    The space-separated list of package names to install

``initial_image``
    Initial image to use for installing base system. Usually it's built from
    ``mirror``/``arch``/``image_release`` triple, so you shouldn't touch it.

``pkgbuilds``
    Directories (relative to project dir) contaning PKGBUILDs to build in
    addition to community packages

``build_dependencies``
    Space-separated list of packages to install for building. Which are removed
    after build finish. Default is ``base-devel`` and in most cases must
    contain at least ``base-devel``. Note: currently all build dependencies
    must be declared here. In future we may install non-optional dependencies
    automatically .

``build_nocheck``
    If set to no-empty string builds with ``makepkg --nocheck``, useful if test
    contain additional dependencies or run too slow.

``additional_repos``
    A space separated list of additional repositories to use. For example::

        arch:
          builder: arch
          parameters:
            additional_repos: archlinuxfr|http://repo.archlinux.fr/$arch
            packages: yaourt

    The name of the repository and url are separated by pipe character.
    The ``$arch`` variable is put into ``pacman.conf`` as is and expanded by
    pacman itself. Note: repository is always added as with
    ``SigLevel = Never`` we may fix this in the future.



Arch-simple
===========

The `arch_simple` builder is a simple builder which just installs packages
using pacman. This backend requires pacman to be installed on the host system,
however, comparing to `arch` builder it can make a smaller container (e.g. you
don't need to have a pacman on guest system).


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
    provision: |
      export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
      export LD_PRELOAD=/tmp/inventory/libfake.so
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
* ``tar``

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

The official ubuntu builder contains has the following features:

* uses official ubuntu image as base (so no need for ``dpkg`` on host)
* standard and universe (multiverse) repository support
* installs packages from PPA's and custom repositories
* installs either using subuid/subgid or as single user (using libfake)
* caches downloaded packages amongst multiple builds

Traditional example for building docs:

.. code-block:: yaml

    builder: ubuntu
    parameters:
      release: precise
      packages: make python-sphinx

More complex example with PPA and "universe":

.. code-block:: yaml

    builder: ubuntu
    uids: [0-1000, 65534]
    gids: [0-1000, 65534]
    parameters:
      repos: universe
      PPAs: hansjorg/rust
      additional_keys: 37FD5E80BD6B6386  # for rust
      packages: make checkinstall rust-0.11

Example with custom repository (note the syntax for repository):

.. code-block:: yaml

    builder: ubuntu
    uids: [0-1000, 65534]
    gids: [0-1000, 65534]
    parameters:
      additional_repos:
        https://get.docker.io/ubuntu|docker|main
      additional_keys: 36A1D7869245C8950F966E92D8576A8BA88D21E9
      packages: lxc-docker


Dependencies
------------

* ``wget`` or ``curl``
* ``tar``
* ``sed``, ``awk``


Parameters
----------

``packages``
    Space separated list of packages to install. May include packages from
    PPA's or custom repos (described below)

``release``
    The ubuntu release name. Default is latest LTS (currently ``trusty``). It
    seems only LTS images are supported by canonical. So only ``precise``
    alternative may work.

``arch``
    The architecture of system. Only ``amd64`` is tested, which is default.

``initial_image``
    The full url of the initial image used to bootstrap system. For most cases
    default is ok (it's constructed from kind/release/arch)

``PPAs``
    The space-separated list of PPA names to use. Packages set in ``packages``
    parameter with be searched for in these PPA's too. The PPA specified as
    ``login/repo`` (e.g. ``hansjorg/rust``).

``repos``
    The space-separated list of repositories to enable (probably only useful
    values are ``universe`` and ``multiverse``)

``additional_repos``
    The space-separated list of additional repos to use. It's in the form of
    ``uri|suite|component1|component2...``. I.e. it's similar to what is used
    in ``sources.list`` but with pipe as separator char and without ``deb``
    prefix. For example, to use docker repository you should use the following
    line (no, I don't know why you need docker in vagga, just example)::

      additional_repos:
        https://get.docker.io/ubuntu|docker|main

``additional_keys``
    The space-separated list of keys to import. You usually need this for using
    PPA's or custom repositories. All keys are imported from
    ``keyserver.ubuntu.com``.

``initial_packages``
    List of packages to install before anything else. Mostly needed for plugins
    for ``apt-get``. Some are detected automatically. You should avoid this
    setting if possible.

``kind``
    The kind of image used as base. Default is ``core`` which means
    ``ubuntu-core`` image is used.



Alpine Linux
============

``alpine`` builder installs `Alpine linux`_ packages. This distribution known
for it's smallest package sizes. Also unlike some other distributions Alpine
has easily downloadable static build of it's package manager, so you don't need
to have ``apk`` (the package manager) installed on host system.

Example:

.. code-block:: yaml

  alpine:
    builder: alpine
    parameters:
      packages: py-sphinx make


To give you some notion of how smaller alpine linux is. This example has size
of about 64Mb. Similar example built by `Debian Debootstrap`_ builder has
size of about 297Mb.


Dependencies
------------

* ``wget`` or ``curl``
* ``tar``


Parameters
----------

``packages``
    Space-separated list of packages (default ``alpine-base``)

``mirror``
    The url of the alpine mirror for installation (default
    ``http://nl.alpinelinux.org/alpine/``)


.. _`Alpine linux`: http://alpinelinux.org/


Node Package Manager
====================

The ``npm`` builder, builds small system with installed npm_ packages. This
is useful for web projects which need nodejs to build some static scripts but
don't need it for other tasks. Here is a encouraging example:

.. code-block:: yaml

   containers:
     react:
       builder: npm
       uids: [0-50, 65534]
       gids: [0-50, 65534]
       parameters:
         packages: react-tools

   commands:
     build:
       container: react
       command: make

This way just typing ``vagga make`` in a project directory when frist run
creates container with react-tools (i.e. ``jsx`` command) and runs ``make``
tool to build whatever is specified in ``Makefile``.

.. warning:: Specifying ``uids`` and ``gids`` is mandatory, as npm is not
   smart enough to skip non-existing users. So you must have ``newuidmap`` and
   ``newgidmap`` installed (from package ``shadow>=4.1`` or ``uidmap``). Also
   you must have something like the following in your system config files
   (assuming your user is ``username`` and your uid is ``1000``)::

        # /etc/subuid
        username:100000:100
        # /etc/subgid
        username:100000:100

   See man subuid(5) and subgid(5) for more info.


Currently installed in container by default are: ``nodejs``, ``npm``, ``make``,
and ``git``. Latter one is mostly needed to install some nodejs packages. And
``make`` is often useful to build javascripts. Container with base system
occupy about 40Mb without additional node modules.

.. note:: Currently we use `Alpine linux`_ to build container. But you should
   not rely this. The only guaranteed is existence of node and other tools
   mentioned above. We may change the base system if feel it reasonable.


Dependencies
------------

* ``wget`` or ``curl``
* ``tar``


Parameters
----------

``packages``
    A space-separated list of packages. Each name may be any string supported
    by ``npm install``

``alpine_packages``
    Space-separated list of alpine packages to install. Usage of this option
    is discouraged. This option may stop working at any moment. Use on your own
    risk.

``alpine_mirror``
    The mirror to use for fetching packages. Usage of this option is
    discouraged. This option may stop working at any moment.



.. _npm: http://npmjs.org


