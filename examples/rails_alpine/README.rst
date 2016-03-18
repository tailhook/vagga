========================
Building a Rails project
========================

This example will show how to create a simple Rails project using vagga.

* `Creating the project structure`_


Creating the project structure
==============================

First, let's create a directory for our new project::

    $ mkdir -p ~/projects/vagga-rails-tutorial && cd ~/projects/vagga-rails-tutorial

Now we need to create our project's structure, so let's create a new container
and tell it to do so.

Create the ``vagga.yaml`` file and add the following to it:

.. code-block:: yaml

    containers:
      app:
        setup:
        - !Alpine v3.3
        - !Install [libxml2, libxslt, zlib] ❶
        - !BuildDeps [libxml2-dev, libxslt-dev, zlib-dev] ❶
        - !Env
          NOKOGIRI_USE_SYSTEM_LIBRARIES: 1 ❷
        - !GemInstall [rails] ❸
        environ:
          HOME: /tmp ❹

* ❶ -- ``rails`` depends on `nokogiri`_, which needs these libs during build and
  runtime.
* ❷ -- ``nokogiri`` ships its own versions of ``libxml2`` and ``libxslt`` in order
  to make it easier to build, but here we are instructing it to use the
  versions provided by Alpine. Refer to `nokogiri docs`_ for details.
* ❸ -- tell ``gem`` to install ``rails``.
* ❹ -- ``rails`` will complain if we do not have a ``$HOME``.

.. _nokogiri: http://www.nokogiri.org
.. _nokogiri docs: http://www.nokogiri.org/tutorials/installing_nokogiri.html

And now run::

    $ vagga _run app rails new . --skip-bundle

This will create a new rails project in the current directory. The ``--skip-bundle``
flag tells rails to not run ``bundle install``, but don't worry, vagga will also
run it for us.

Now that we have our rails project, let's change our container to use the
``Gemfile`` instead of installing gems manually:

.. code-block:: yaml

    containers:
      app:
        setup:
        - !Alpine v3.3
        - !Install [libxml2, libxslt, zlib, sqlite-libs, nodejs] ❶
        - !BuildDeps [libxml2-dev, libxslt-dev, zlib-dev, sqlite-dev] ❶
        - !Env
          NOKOGIRI_USE_SYSTEM_LIBRARIES: 1
        - !GemBundle ❷
        environ:
          HOME: /tmp

* ❶ -- we need ``sqlite`` for the development database and ``nodejs`` for the
  asset pipeline (specifically, the ``uglifier`` gem).
* ❷ -- install dependencies from ``Gemfile`` using ``bundler``.

Before we test our project, let's add two gems into the ``Gemfile``:

.. code-block:: ruby

    # Gemfile
    # ...
    gem 'bigdecimal'
    gem 'tzinfo-data'
    # ...

Without these two gems, you may run into import errors.

To test if everything is Ok, let's run our rails project::

    $ vagga _run app rails server

Now visit ``localhost:3000`` to see rails default page.
