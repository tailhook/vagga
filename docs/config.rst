=============
Configuration
=============

Main vagga configration file is ``vagga.yaml`` it's usually in the root of the
project dir. It can also be in ``.vagga/vagga.yaml`` (but it's not recommended).

The ``vagga.yaml`` has two sections:

* ``containers`` -- description of the containers
* ``commands`` -- a set of commands defined for the project

.. toctree::
   :maxdepth: 2

   container_params
   commands
   build_commands
   build_steps
   volumes
   upgrading
   supervision
   pid1mode

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

.. _YAML: http://yaml.org
