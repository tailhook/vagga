==========================
Building a Laravel project
==========================

This example will show how to create a simple Laravel project using vagga.

* `Creating the project structure`_


Creating the project structure
==============================

In order to create the initial project structure, we will need a container with
the Laravel installer. First, let's create a directory for our project::

    $ mkdir -p ~/projects/vagga-laravel-tutorial && cd ~/projects/vagga-laravel-tutorial

Create the ``vagga.yaml`` file and add the following to it:

.. code-block:: yaml

    containers:
      laravel:
        setup:
        - !Alpine v3.3
        - !ComposerInstall [laravel/installer]

And then run::

    $ vagga _run laravel laravel new src
    $ mv src/* src/.* .
    $ rmdir src

We want our project's files in the current directory (the one containing
``vagga.yaml``) but Laravel installer only accepts an empty directory, so we tell
it to create out project into ``src``, move its contents into the current directory
and remove it.

You may see in the console ``sh: composer: not found`` because Laravel installer
is trying to run ``composer install``, but don't worry about it, vagga will take
care of that for us.

Now change our container to install dependencies from ``composer.json``

.. code-block:: yaml

    containers:
      laravel:
        setup:
        - !Alpine v3.3
        - !Sh |
            if [ ! -f .env ]; then
              cp .env.example .env
              php artisan key:generate
            fi
        - !ComposerDependencies

.. warning:: Your composer dependencies will not be installed at the ``./vendor``
  directory. Instead, the are installed globally at ``/usr/local/lib/composer/vendor``,
  so be sure to require ``autoload.php`` from there.

Requiring the right autoload.php
--------------------------------

**THIS IS VERY IMPORTANT!**

Before doing anything with our project, we need to require the right ``autoload.php``.
First, let's set an environment variable to help us out:

.. code-block:: yaml

    containers:
      laravel:
        environ: &env
          ENV_CONTAINER: 1
        setup:
        - !Alpine v3.3
        - !Sh |
            if [ ! -f .env ]; then
              cp .env.example .env
              php artisan key:generate
            fi
        - !Env
          <<: *env
        - !ComposerDependencies

Setting this variable will help us tell whether we're running inside a container
or not. This is particularly useful if we deploy our project to a shared server.

Now open ``bootstrap/autoload.php`` and change the line
``require __DIR__.'/../vendor/autoload.php';`` as follows:

.. code-block:: php

    <?php
    // ...
    if (getenv('ENV_CONTAINER') === false) {
        require __DIR__.'/../vendor/autoload.php';
    } else {
        require '/usr/local/lib/composer/vendor/autoload.php';
    }
    // ...

Running the project
-------------------

To test if everything is ok, let's add a command to run our project:

.. code-block:: yaml

    containers:
      # ...
    commands:
      run: !Command
        container: laravel
        description: run the laravel development server
        run: php artisan serve

Now run::

    $ vagga run

And visit ``localhost:8000``. If everithing was fine, you will see Laravel default
page saying "Laravel 5".
