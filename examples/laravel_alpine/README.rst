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
``vagga.yaml``) but Laravel installer only accepts an empty directory, so we
tell it to create out project into ``src``, move its contents into the current
directory and remove ``src``.

You may see in the console ``sh: composer: not found`` because Laravel installer
is trying to run ``composer install``, but don't worry about it, vagga will take
care of that for us.

Now there are 3 steps we need to follow:

1. Install dependencies from ``composer.json``
2. Ensure ``.env`` exists and application key is generated
3. Require the right ``autoload.php``

Installing from ``composer.json``
---------------------------------

This is the easy part. Just change our container as follows:

.. code-block:: yaml

    containers:
      laravel:
        setup:
        - !Alpine v3.3
        - !ComposerDependencies

Setup ``.env`` and application key
----------------------------------

Laravel uses `dotenv`_ to load configuration into environment automatically from
a ``.env`` file in development. So, during container building, we will create a
minimal ``.env`` and call ``php artisan key:generate`` to generate the
application key.

Now change our container as follows:

.. code-block:: yaml

    containers:
      laravel:
        setup:
        - !Alpine v3.3
        - !Text
          /work/.env: |
              APP_ENV=local
              APP_DEBUG=true
              APP_KEY=SomeRandomString
              APP_URL=http://localhost
        - !ComposerDependencies
        - !Sh php artisan key:generate

.. _dotenv: https://github.com/vlucas/phpdotenv

Requiring the right autoload.php
--------------------------------

.. warning:: Your composer dependencies will not be installed at the ``./vendor``
  directory. Instead, the are installed globally at ``/usr/local/lib/composer/vendor``,
  so be sure to follow this section to see how to require ``autoload.php`` from
  the right location.

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
        - !Env { <<: *env }
        - !Text
          /work/.env: |
              APP_ENV=local
              APP_DEBUG=true
              APP_KEY=SomeRandomString
              APP_URL=http://localhost
        - !ComposerDependencies
        - !Sh php artisan key:generate

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

This will enable our project to be run either from a container (as we are doing
here with vagga) or from a shared server.

.. note:: If you are deploying your project to production using a container, you
  can just ``require '/usr/local/lib/composer/vendor/autoload.php';`` and ignore
  the environment variable we just set.

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

And visit ``localhost:8000``. If everithing was fine, you will see Laravel
default page saying "Laravel 5".
