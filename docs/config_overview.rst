========
Overview
========

The ``vagga.yaml`` has two sections:

* ``containers`` -- description of the containers
* ``commands`` -- a set of commands defined for the project

There is also two top-level options:

.. opt:: mixins
   **This functionality is experimental**. Some details can change in future.

   This is a list of vagga configs that will be "mixed in" into current config.
   This basically means that we import all the commands and containers from
   them literally.

   When adding mixins, latter one overrides commands and containers in
   the former configs. And the ones in ``vagga.yaml`` override all the mixins.

   There are a few use-cases for mixins:

   1. Splitting config into several groups of things, while putting together
      containers and commands (latter contrasts to using includes_).
   2. Use a generated parts of configs. Because non-existing or invalid mixins
      are ignored (with a warning) you can generate or update mixins by vagga
      commands without risk of making defunct vagga config.
   3. Use vagga config from a subproject (but be aware that paths resolve to
      original ``vagga.yaml``, not the included one)
   4. Override things from git-commited ``vagga.yaml`` to custom one (note the
      latter requires not to commit ``vagga.yaml`` itself, but only mixed in
      things)

   .. versionadded:: 0.7.1

   .. versionchanged:: 0.8.1

      There are implicit mixins:

      1. ``vagga.local.yaml``
      2. ``.vagga.local.yaml`` (hidden)
      3. ``.vagga/local.yaml`` (in vagga dir)

      They should be used to add or override local commands which shouldn't be
      committed to a central repository. They work similarly as normal mixins
      and can contain additional mixins themselves.


   .. _includes: http://rust-quire.readthedocs.io/en/latest/user.html#includes



.. opt:: minimum-vagga

   (default is no limit) Defines minimum version to run the configuration file.
   If you put::

        minimum-vagga: v0.5.0

   Into ``vagga.yaml`` other users will see the following error::

        Please upgrade vagga to at least "v0.5.0"

   This is definitely optional, but useful if you start using new features, and
   want to communicate the version number to a team. Versions from testing
   work as well. To see your current version use::

        $ vagga --version


.. _containers:

Containers
==========

Example of one container defined:

.. code-block:: yaml

   containers:
     sphinx:
       setup:
       - !Ubuntu xenial
       - !Install [python3-sphinx, make]

The YAML above defines a container named ``sphinx``, which is built with two
steps: download and unpack ubuntu ``xenial`` base image, and install packages
name ``python-sphinx, make``  inside the container.


Commands
========

Example of command defined:

.. code-block:: yaml

   commands:
     build-docs: !Command
       description: Build vagga documentation using sphinx
       container: sphinx
       work-dir: docs
       run: [make]

The YAML above defines a command named ``build-docs``, which is run in
container named ``sphinx``, that is run in ``docs/`` sub dir of project, and
will run command ``make`` in container. So running::

    $ vagga build-docs html

Builds html docs using sphinx inside a container.

See commands_ for comprehensive description of how to define commands.
