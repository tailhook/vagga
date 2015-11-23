========
Overview
========

The ``vagga.yaml`` has two sections:

* ``containers`` -- description of the containers
* ``commands`` -- a set of commands defined for the project

There is also additional top-level option:

.. opt:: minimum-vagga

   (default is no limit) Defines minimum version to run the configuration file.
   If you put::

        minimum-vagga: v0.4.2

   Into ``vagga.yaml`` other users will see the following error::

        Please upgrade vagga to at least "v0.4.2"

   This is definitely optional, but useful if you start using new features, and
   want to communicate the version number to a team. Versions from testing
   work as well. To see your current version use::

        vagga --version


.. _containers:

Containers
==========

Example of one container defined:

.. code-block:: yaml

  containers:
    sphinx:
      setup:
      - !Ubuntu trusty
      - !Install [python-sphinx, make]

The YAML above defines a container named ``sphinx``, which is built with two
steps: download and unpack ubuntu ``trusty`` base image, and install install
packages name ``python-sphinx, make``  inside the container.


Commands
========

Example of command defined:

.. code-block:: yaml

   commands:
     build-docs: !Command
       description: Build vagga documentation using sphinx
       container: sphinx
       work-dir: docs
       run: make

The YAML above defines a command named ``build-docs``, which is run in
container named ``sphinx``, that is run in ``docs/`` sub dir of project, and
will run command ``make`` in container. So running::

    > vagga build-docs html

Builds html docs using sphinx inside a container.

See commands_ for comprehensive description of how to define commands.
