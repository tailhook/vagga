===========================
What Makes Vagga Different?
===========================

There are four prominent features of vagga:

* Command-centric workflow instead of container-centric
* Lazy creation of containers
* Containers are versioned and automatically rebuilt
* Running multiple processes without headache

Let's discuss them in details


Command-Centric Workflow
========================

When you start working on project, you don't need to know anything about
virtual machines, dependencies, paths whatever. You just need to know what you
can do with it.

Consider we have an imaginary web application. Let's see what we can
do::

    > git clone git@git.git:somewebapp.git somewebapp
    > cd somewebapp
    > vagga
    Available commands:
        build-js    build javascript files needed to run application
        serve       serve a program on a localhost

Ok, now we know that we probably expected to build javascipt files and that we
can run a server. We now just do::

    > vagga build-js
    # container created, dependencies populated, javascripts are built
    > vagga serve
    Now you can go to http://localhost:8000 to see site in action

Compare that to vagrant::

    > vagrant up
    # some machine(s) created
    > vagrant ssh
    # now you are in new shell. What to do?
    > make
    # ok probably something is built (if project uses make), what now?
    > less README
    # long reading follows

Or compare that to docker::

    > docker pull someuser/somewebapp
    > docker run --rm --it someuser/somewebapp
    # if you are lucky something is run, but how to build it?
    # let's see the README


Lazy Container Creation
=======================

There are few interesting cases where lazy containers help.


Application Requires Multiple Environments
------------------------------------------

In our imaginary web application described above we might have very different
environments to build javascript files, and to run the application. For example
javascripts are usually built and compressed using Node.js. But if our server
is written in python we don't need Node.js to run application. So it's often
desirable to run application in a container without build dependencies, at
least to be sure that you don't miss some dependency.

Let's declare that with vagga. Just define two containers:

.. code-block:: yaml

   containers:

     build:
       builder: ubuntu
       parameters:
         packages: make nodejs uglifyjs

     serve:
       builder: ubuntu
       parameters:
         packages: python-django

One for each command:

.. code-block:: yaml

   commands:

     build-js:
       container: build
       run: "make build-js"

     serve:
       container: serve
       run: "python manage.py runserver"

Similarly might be defined test container and command:

.. code-block:: yaml

   containers:

     testing:
       builder: ubuntu
       parameters:
         packabes: make nodejs uglifyjs python-django nosetests

   commands:

     test:
       container: testing
       command: nosetests

And your user never care how many containers are there. User only runs whatever
comands he needs.

How is it done in vagrant?

::

    > vagrant up
    # two containers are up at this point
    > vagrant ssh build -- make
    # built, now we don't want to waste memory for build virtual machine
    > vagrant halt build
    > vagrant ssh serve -- python manage.py runserver


Project With Examples
---------------------

Many open-source projects and many proprietary libraries have some examples.
Often samples have additional dependencies. If you developing a markdown parser
library, you might have a tiny example web application using flask that
converts markdown to html on the fly::

    > vagga
    Available commands:
        md2html         convert markdown to html without installation
        tests           run tests
        example-web     run live demo (flask app)
        example-plugin  example of plugin for markdown parser
    > vagga example-web
    Now go to http://localhost:8000 to see the demo

How would you achieve the same with vagrant?

::

    > ls -R examples
    examples/web:
    Vagrantfile README flask-app.py

    examples/plugin:
    Vagrantfile README main.py plugin.py

    > cd examples/web
    > vagrant up && vagrant ssh -- python main.py --help
    > vagrant ssh -- python main.py --port 8000
    # ok got it, let's stop it
    > vagrant halt && vagrant destroy

I.e. a ``Vagrantfile`` per example. Then user must keep track of what
containers he have done ``vagrant up`` in, and do not forget to shutdown and
destroy them.

.. note:: example with Vagrant is very imaginary, because unless you insert
   files in container on provision stage, your project root is inaccessible in
   container of ``examples/web``. So you need some hacks to make it work.

Docker case is very similar to Vagrant one.


Container Versioning and Rebuilding
===================================

What if the project dependencies are changed by upstream? No problem::

    > git pull
    > vagga serve
    # vagga notes that dependencies changed, and rebuilds container
    > git checkout stable
    # moving to stable branch, to fix some critical bug
    > vagga serve
    # vagga uses old container that is probably still around

Vagga hashes dependencies, and if the hash changed creates new container.
Old ones are kept around for a while, just in case you revert to some older
commit or switch to another branch.

.. note:: For all backends except ``nix``, version hash is derived from
   parameters of a builder. For ``nix`` we use hash of nix derivations that is
   used to build container, so change in ``.nix`` file or its dependencies
   trigger rebuild too (unless it's non-significant change, like whitespace
   change or swapping lines).

How you do this with Vagrant::

    > git pull
    > vagrant ssh -- python manage.py runserver
    ImportError
    > vagrant reload
    > vagrant ssh -- python manage.py runserver
    ImportError
    > vagrant reload --provision
    #  If you are lucky and your provision script is good, dependency installed
    > vagrant ssh -- python manage.py runserver
    # Ok it works
    > git checkout stable
    > vagrant ssh -- python manage.py runserver
    # Wow, we still running dependencies from "master", since we added
    # a dependency it works for now, but may crash when deploying
    > vagrant restart --provision
    # We used ``pip install requirements.txt`` in provision
    # and it doesn't delete dependencies
    > vagrant halt
    > vagrant destroy
    > vagrant up
    # let's wait ... it sooo long.
    > vagrant ssh -- python manage.py runserver
    # now we are safe
    > git checkout master
    # Oh no, need to rebuild container again?!?!

Using Docker? Let's see::

    > git pull
    > docker run --rm -it me/somewebapp python manage.py runserver
    ImportError
    > docker tag me/somewebapp:latest me/somewebapp:old
    > docker build -t me/somewebapp .
    > docker run --rm -it me/somewebapp python manage.py runserver
    # Oh, that was simple
    > git checkout stable
    > docker run --rm -it me/somewebapp python manage.py runserver
    # Oh, crap, I forgot to downgrade container
    # We were smart to tag old one, so don't need to rebuild:
    > docker run --rm -it me/somewebapp:old python manage.py runserver
    # Let's also rebuild dependencies
    > ./build.sh
    Running: docker run --rm me/somewebapp_build python manage.py runserver
    # Oh crap, we have hard-coded container name in build script?!?!

Well, docker is kinda easier because we can have multiple containers around,
but still hard to get right.


Running Multiple Processes
==========================

Many projects require multiple processes around. E.g. when running web
application on development machine there are at least two components: database
and app itself. Usually developers run database as a system process and a
process in a shell.

When running in production one usually need also a cache and a webserver. And
developers are very lazy to run those components on development system, just
because it's complex to manage. E.g. if you have a startup script like this::

    #!/bin/sh
    redis-server ./config/redis.conf &
    python manage.py runserver

You are going to loose ``redis-server`` running in background when python
process dead or interrupted. Running them in different tabs of your terminal
works while there are two or three services. But today more and more projects
adopt service-oriented architecture. Which means there are many services in
your project (e.g. in our real-life example we had 11 services written by
ourselves and we also run two mysql and two redis nodes to emulate clustering).

This means either production setup and development are too diverse, or we need
better tools to manage processes.

How vagrant helps? Almost in no way. You can run some services as a system
services inside a vagrant. And you can also have multiple virtual machines
with services, but this doesn't solve core problem.

How docker helps? It only makes situation worse, because now you need to follow
logs of many containers, and remember to ``docker stop`` and ``docker rm`` the
processes on every occassion.

Vagga's way:

.. code-block:: yaml

  commands:
    run_full_app:
      supervise-mode: stop-on-failure
      supervise:
        web:
          container: python
          run: "python manage.py runserver"
        redis:
          container: redis
          run: "redis-server ./config/redis.conf"
        celery:
          container: python
          run: "python manage.py celery worker"

No just run::

    > vagga run_full_app
    # two python processes and a redis started here

It not only allows you to start processes in multiple containers, it also
does meaningful monitoring of them. The ``stop-on-failure`` mode means if any
process failed to start or terminated, terminate all processes. It's opposite
to the usual meaning of supervising, but it's super-useful development tool.

Let's see how it's helpful. In example above celery may crash (for example
because of misconfiguration, or OOM, or whatever). Usually when running many
services you have many-many messages on startup, so you may miss it. Or it may
crash later. So you click on some task in web app, and wait when the task is
done. After some time, you think that it *may* be too long, and start looking
in logs here and there. And after some tinkering around you see that celery is
just down. Now, you lost so much time just waiting. Wouldn't it be nice if
everything is just crashed and you notice it immediately? Yes it's what
``stop-on-failure`` does.

Then if you want to stop it, you just press ``Ctrl+C`` and wait for it to shut
down. If it hangs for some reason (may be you created a bug), you repeat or
press ``Ctrl+/`` (which is ``SIGQUIT``), or just do ``kill -9`` from another
shell. In any case vagga will not exit until all processes are shut down and
no hanging processes are left ever (Yes, even with ``kill -9``).


