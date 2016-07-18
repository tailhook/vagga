===========
Entry Point
===========

Vagga is a tool to create development environments. In particular it is
able to:

* Build container and run program with single command, right after "git pull"
* Automatically rebuild container if project dependencies change
* Run multiple processes (e.g. application and database) with single command
* Execute network tolerance tests

All this seamlessly works using linux namespaces (or containers).

Example
=======

Let's make config for hello-world flask_ application. To start you need to put
following in ``vagga.yaml``:

.. code-block:: yaml

    containers:
      flask: ❶
        setup:
        - !Ubuntu xenial ❷
        - !UbuntuUniverse ❸
        - !Install [python3-flask] ❹
    commands:
      py3: !Command ❺
        container: flask ❻
        run: python3 ❼

* ❶ -- create a container "flask"
* ❷ -- install base image of ubuntu
* ❸ -- enable the universe repository in ubuntu
* ❹ -- install flask from package (from ubuntu universe)
* ❺ -- create a simple command "py3"
* ❻ -- run command in container "flask"
* ❼ -- the command-line is "python3"

To run command just run ``vagga command_name``:

.. code-block:: bash

    $ vagga py3
    [ .. snipped container build log .. ]
    Python 3.5.1+ (default, Mar 30 2016, 22:46:26)
    [GCC 5.3.1 20160330] on linux
    Type "help", "copyright", "credits" or "license" for more information.
    >>> import flask
    >>>

This is just a lazy example. Once your project starts to mature you want to
use some specific version of flask and some other dependencies:

.. code-block:: yaml

    containers:
      flask:
        setup:
        - !Ubuntu xenial
        - !Py3Install
          - werkzeug==0.9.4
          - MarkupSafe==0.23
          - itsdangerous==0.22
          - jinja2==2.7.2
          - Flask==0.10.1
          - sqlalchemy==0.9.8

And if another developer does ``git pull`` and gets this config, running
``vagga py3`` next time will rebuild container and run command in the new
environment without any additional effort:

.. code-block:: bash

    $ vagga py3
    [ .. snipped container build log .. ]
    Python 3.5.1+ (default, Mar 30 2016, 22:46:26)
    [GCC 5.3.1 20160330] on linux
    Type "help", "copyright", "credits" or "license" for more information.
    >>> import flask, sqlalchemy
    >>>

.. note:: Container is rebuilt from scratch on each change. So *removing*
   package works well. Vagga also uses smart caching of packages to make
   rebuilds fast.

You probably want to move python dependencies into ``requirements.txt``:

.. code-block:: yaml

    containers:
      flask:
        setup:
        - !Ubuntu xenial
        - !Py3Requirements "requirements.txt"

And vagga is smart enough to rebuild if ``requirements.txt`` change.

----

In case you've just cloned the project you might want to run bare ``vagga`` to
see which commands are available. For example, here are some commands available
in vagga project itself::

    $ vagga
    Available commands:
        make                Build vagga
        build-docs          Build vagga documentation
        test                Run self tests

(the descriptions on the right are added using ``description`` key in command)


.. _flask: http://flask.pocoo.org/docs/0.10/


More Reading
============

* `Managing Dependencies with Vagga <https://medium.com/@paulcolomiets/managing-dependencies-with-vagga-79181046db66>`_
  shows basic concepts of using vagga and what problems it solves.

* `The Higher Level Package Manager <https://medium.com/@paulcolomiets/vagga-the-higher-level-package-manager-e49e85fed42a>`_ -- discussion of vagga goals and future

* `Evaluating Mesos <https://medium.com/@paulcolomiets/evaluating-mesos-4a08f85473fb>`_
  discuss how to run network tolerance tests.
