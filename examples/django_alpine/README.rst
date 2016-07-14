=========================
Building a Django project
=========================

This example will show how to create a simple Django project using vagga.

* `Creating the project structure`_
* `Freezing dependencies`_
* `Let's add a dependency`_
* `Adding some code`_
* `Trying out memcached`_
* `Why not Postgres?`_
* `Making Postgres data persistent`_


Creating the project structure
==============================

In order to create the initial project structure, we will need a container with Django
installed. First, let's create a directory for our project::

    $ mkdir -p ~/projects/vagga-django-tutorial && cd ~/projects/vagga-django-tutorial

Now create the ``vagga.yaml`` file and add the following to it:

.. code-block:: yaml

    containers:
      django:
        setup:
        - !Alpine v3.4
        - !Py3Install ['Django >=1.9,<1.10']

and then run::

    $ vagga _run django django-admin startproject MyProject .

This will create a project named ``MyProject`` in the current directory. It will
look like::

    ~/projects/vagga-django-tutorial
    ├── manage.py
    ├── MyProject
    │   ├── __init__.py
    │   ├── settings.py
    │   ├── urls.py
    │   └── wsgi.py
    └── vagga.yaml

Notice that we used ``'Django >=1.9,<1.10'`` instead of just ``Django``. It is a
good practice to always specify the major and minor versions of a dependency.
This prevents an update to an incompatible version of a library breaking you project.
You can change the Django version if there is a newer version available
(``'Django >=1.10,<1.11'`` for instance).

Freezing dependencies
=====================

It is a common practice for python projects to have a ``requirements.txt`` file
that will hold the exact versions of the project dependencies. This way, any
developer working on the project will have the same dependencies.

In order to generate the ``requirements.txt`` file, we will create another
container called ``app-freezer``, which will list our project's dependencies and
output the requirements file.

.. code-block:: yaml

    containers:
      app-freezer: ❶
        setup:
        - !Alpine v3.4
        - !Py3Install
          - pip ❷
          - 'Django >=1.9,<1.10'
        - !Sh pip freeze > requirements.txt ❸
      django:
        setup:
        - !Alpine v3.4
        - !Py3Requirements requirements.txt ❹

* ❶ -- our new container
* ❷ -- we need pip available to freeze dependencies
* ❸ -- generate the requirements file
* ❹ -- just reference the requirements file from ``django`` container

Every time we add a new dependency, we need to rebuild the ``app-freezer``
container to generate the updated ``requirements.txt``.

Now, build the ``app-freezer`` container::

    $ vagga _build app-freezer

You will notice the new ``requirements.txt`` file holding a content similar to::

    Django==1.9.7

And now let's run our project. Edit ``vagga.yaml`` to add the ``run`` command:

.. code-block:: yaml

    containers:
      # same as before
    commands:
      run: !Command
        description: Start the django development server
        container: django
        run: python3 manage.py runserver

and then run::

    $ vagga run

If everything went right, visiting ``localhost:8000`` will display Django's
welcome page saying 'It worked!'.

Let's add a dependency
======================

By default, Django is configured to use sqlite as its database, but we want to
use a database url from an environment variable, since it's more flexible.
However, Django does not understand database urls, so we need ``dj-database-url``
to convert the database url into what Django understand.

Add ``dj-database-url`` to our ``app-freezer`` container:

.. code-block:: yaml

    containers:
      app-freezer:
        setup:
        - !Alpine v3.4
        - !Py3Install
          - pip
          - 'Django >=1.9,<1.10'
          - 'dj-database-url >=0.4,<0.5'
        - !Sh pip freeze > requirements.txt

Rebuild the ``app-freezer`` container to update ``requirements.txt``::

    $ vagga _build app-freezer

Set the environment variable:

.. code-block:: yaml

    containers:
      #...
      django:
        environ:
          DATABASE_URL: sqlite:///db.sqlite3 ❶
        setup:
        - !Alpine v3.4
        - !Py3Requirements requirements.txt

* ❶ -- will point to /work/db.sqlite3

Now let's change our project's settings by editing ``MyProject/settings.py``:

.. code-block:: python

    # MyProject/settings.py
    import os
    import dj_database_url

    # other settings

    DATABASES = {
        # will read DATABASE_URL from environment
        'default': dj_database_url.config()
    }

Let's another shortcut command for ``manage.py``:

.. code-block:: yaml

    commands:
      # ...
      manage.py: !Command
        description: Shortcut to manage.py
        container: django
        run: [python3, manage.py]

.. note:: This command accept arguments by default, so
   instead of writing it long ``vagga _run django python3 manage.py runserver``
   we will be able to shorten it to ``vagga manage.py runserver``

To see if it worked, let's run the migrations from the default Django apps and
create a superuser::

    $ vagga manage.py migrate
    $ vagga manage.py createsuperuser

After creating the superuser, run our project::

    $ vagga run

visit ``localhost:8000/admin`` and log into the Django admin.

Adding some code
================

Before going any further, let's add a simple app to our project.

First, start an app called 'blog'::

    $ vagga manage.py startapp blog

Add it to ``INSTALLED_APPS``:

.. code-block:: python

    # MyProject/settings.py
    INSTALLED_APPS = [
        # ...
        'blog',
    ]

Create a model:

.. code-block:: python

    # blog/models.py
    from django.db import models


    class Article(models.Model):
        title = models.CharField(max_length=100)
        body = models.TextField()

Create the admin for our model:

.. code-block:: python

    # blog/admin.py
    from django.contrib import admin
    from .models import Article


    @admin.register(Article)
    class ArticleAdmin(admin.ModelAdmin):
        list_display = ('title',)

Create and run the migration::

    $ vagga manage.py makemigrations
    $ vagga manage.py migrate

Run our project::

    $ vagga run

And visit ``localhost:8000/admin`` to see our new model in action.

Now create a couple views:

.. code-block:: python

    # blog/views.py
    from django.views import generic
    from .models import Article


    class ArticleList(generic.ListView):
        model = Article
        paginate_by = 10


    class ArticleDetail(generic.DetailView):
        model = Article

Create the templates:

.. code-block:: django

    {# blog/templates/blog/article_list.html #}
    <!DOCTYPE html>
    <html>
    <head>
      <title>Article List</title>
    </head>
    <body>
      <h1>Article List</h1>
      <ul>
      {% for article in article_list %}
        <li><a href="{% url 'blog:article_detail' article.id %}">{{ article.title }}</a></li>
      {% endfor %}
      </ul>
    </body>
    </html>

.. code-block:: django

    {# blog/templates/blog/article_detail.html #}
    <!DOCTYPE html>
    <html>
    <head>
      <title>Article List</title>
    </head>
    <body>
      <h1>{{ article.title }}</h1>
      <p>
        {{ article.body }}
      </p>
    </body>
    </html>

Set the urls:

.. code-block:: python

    # blog/urls.py
    from django.conf.urls import url
    from . import views

    urlpatterns = [
        url(r'^$', views.ArticleList.as_view(), name='article_list'),
        url(r'^(?P<pk>\d+?)$', views.ArticleDetail.as_view(), name='article_detail'),
    ]

.. code-block:: python

    # MyProject/urls.py
    from django.conf.urls import url, include
    from django.contrib import admin

    urlpatterns = [
        url(r'^', include('blog.urls', namespace='blog')),
        url(r'^admin/', admin.site.urls),
    ]

Now run our project::

    $ vagga run

and visit ``localhost:8000``. Try adding some articles through the admin to see
the result.

Trying out memcached
====================

Many projects use `memcached <http://memcached.org/>`_ to speed up things, so
let's try it out.

Add ``pylibmc`` and ``django-cache-url`` to our ``app-freezer``, as well as the
build dependencies of ``pylibmc``:

.. code-block:: yaml

    containers:
      app-freezer:
        setup:
        - !Alpine v3.4
        - &build_deps !BuildDeps ❶
          - libmemcached-dev ❷
          - zlib-dev ❷
        - !Py3Install
          - pip
          - 'Django >=1.9,<1.10'
          - 'dj-database-url >=0.4,<0.5'
          - 'pylibmc >=1.5,<1.6'
          - 'django-cache-url >=1.0,<1.1' ❸
        - !Sh pip freeze > requirements.txt
      django:
        environ:
          DATABASE_URL: sqlite:///db.sqlite3
        setup:
        - !Alpine v3.4
        - *build_deps ❹
        - !Py3Requirements requirements.txt

* ❶ -- we used an YAML anchor (``&build_deps``) to avoid repetition of the
  build dependencies
* ❷ -- libraries needed to build pylibmc
* ❸ -- used to configure the cache through an url
* ❹ -- the YAML alias ``*build_deps`` references the anchor declared in the
  ``app-freezer`` container, so we don't need to repeat the build dependencies
  on both containers

And rebuild the container::

    $ vagga _build app-freezer

Add the ``pylibmc`` runtime dependencies to our ``django`` container:

.. code-block:: yaml

    containers:
      # ...
      django:
        setup:
        - !Alpine v3.4
        - *build_deps
        - !Install
          - libmemcached ❶
          - zlib ❶
          - libsasl ❶
        - !Py3Requirements requirements.txt
        environ:
          DATABASE_URL: sqlite:///db.sqlite3

* ❶ -- libraries needed by pylibmc at runtime

Crate a new container called ``memcached``:

.. code-block:: yaml

    containers:
      # ...
      memcached:
        setup:
        - !Alpine v3.4
        - !Install [memcached]

Create the command to run with caching:

.. code-block:: yaml

    commands:
      # ...
      run-cached: !Supervise
        description: Start the django development server alongside memcached
        children:
          cache: !Command
            container: memcached
            run: memcached -u memcached -vv ❶
          app: !Command
            container: django
            environ:
              CACHE_URL: memcached://127.0.0.1:11211 ❷
            run: python3 manage.py runserver

* ❶ -- run memcached as verbose so we see can see the cache working
* ❷ -- set the cache url

Change ``MyProject/settings.py`` to use our ``memcached`` container:

.. code-block:: python

    import os
    import dj_database_url
    import django_cache_url
    # ...
    CACHES = {
        # will read CACHE_URL from environment
        'default': django_cache_url.config()
    }

Configure our view to cache its response:

.. code-block:: python

    # blog/urls.py
    from django.conf.urls import url
    from django.views.decorators.cache import cache_page
    from . import views

    cache_15m = cache_page(60 * 15)

    urlpatterns = [
        url(r'^$', views.ArticleList.as_view(), name='article_list'),
        url(r'^(?P<pk>\d+?)$', cache_15m(views.ArticleDetail.as_view()), name='article_detail'),
    ]

Now, run our project with memcached::

    $ vagga run-cached

And visit any article detail page, hit ``Ctrl+r`` to avoid browser cache and watch
the memcached output on the terminal.

Why not Postgres?
=================

We can test our project against a Postgres database, which is probably what we
will use in production.

First add ``psycopg2`` and its build dependencies to ``app-freezer``:

.. code-block:: yaml

    containers:
      app-freezer:
        setup:
        - !Alpine v3.4
        - !BuildDeps
          - libmemcached-dev
          - zlib-dev
          - postgresql-dev ❶
        - !Py3Install
          - pip
          - 'Django >=1.9,<1.10'
          - 'dj-database-url >=0.4,<0.5'
          - 'pylibmc >=1.5,<1.6'
          - 'django-cache-url >=1.0,<1.1'
          - 'psycopg2 >=2.6,<2.7' ❷
        - !Sh pip freeze > requirements.txt

* ❶ -- library needed to build psycopg2
* ❷ -- psycopg2 dependency

Rebuild the container::

    $ vagga _build app-freezer

Add the runtime dependencies of ``psycopg2``:

.. code-block:: yaml

    containers:
      django:
        setup:
        - !Alpine v3.4
        - *build_deps
        - !Install
          - libmemcached
          - zlib
          - libsasl
          - libpq ❶
        - !Py3Requirements requirements.txt
        environ:
          DATABASE_URL: sqlite:///db.sqlite3

* ❶ -- library needed by psycopg2 at runtime

Before running our project, we need a way to automatically create our superuser.
We can crate a migration to do this. First, create an app called ``common``::

    $ vagga manage.py startapp common

Add it to ``INSTALLED_APPS``:

.. code-block:: python

    INSTALLED_APPS = [
        # ...
        'common',
        'blog',
    ]

Create the migration for adding the admin user::

    $ vagga manage.py makemigrations -n create_superuser --empty common

Change the migration to add our admin user:

.. code-block:: python

    # common/migrations/0001_create_superuser.py
    from django.db import migrations
    from django.contrib.auth.hashers import make_password


    def create_superuser(apps, schema_editor):
        User = apps.get_model("auth", "User")
        User.objects.create(username='admin',
                            email='admin@example.com',
                            password=make_password('change_me'),
                            is_superuser=True,
                            is_staff=True,
                            is_active=True)


    class Migration(migrations.Migration):

        dependencies = [
            ('auth', '__latest__')
        ]

        operations = [
            migrations.RunPython(create_superuser)
        ]

Create the database container:

.. code-block:: yaml

    containers:
      #..
      postgres:
        setup:
        - !Ubuntu xenial
        - !Install [postgresql]
        - !EnsureDir /data
        environ:
          PGDATA: /data
          PG_PORT: 5433
          PG_DB: test
          PG_USER: vagga
          PG_PASSWORD: vagga
          PG_BIN: /usr/lib/postgresql/9.5/bin
        volumes:
          /data: !Tmpfs
            size: 100M
            mode: 0o700

And then add the command to run with Postgres:

.. code-block:: yaml

    commands:
      run-postgres: !Supervise
        description: Start the django development server using Postgres database
        children:
          app: !Command
            container: django
            environ:
              DATABASE_URL: postgresql://vagga:vagga@127.0.0.1:5433/test
            run: |
                touch /work/.dbcreation # Create lock file
                while [ -f /work/.dbcreation ]; do sleep 0.2; done # Acquire lock
                python3 manage.py migrate
                python3 manage.py runserver
          db: !Command
            container: postgres
            run: |
                chown postgres:postgres $PGDATA;
                su postgres -c "$PG_BIN/pg_ctl initdb";
                su postgres -c "echo 'host all all all trust' >> $PGDATA/pg_hba.conf"
                su postgres -c "$PG_BIN/pg_ctl -w -o '-F --port=$PG_PORT -k /tmp' start";
                su postgres -c "$PG_BIN/psql -h 127.0.0.1 -p $PG_PORT -c \"CREATE USER $PG_USER WITH PASSWORD '$PG_PASSWORD';\""
                su postgres -c "$PG_BIN/createdb -h 127.0.0.1 -p $PG_PORT $PG_DB -O $PG_USER";
                rm /work/.dbcreation # Release lock
                sleep infinity

Now run::

    $ vagga run-postgres

Visit ``localhost:8000/admin`` and try to log in with the user and password we
defined in the migration.

Making Postgres data persistent
-------------------------------

It is possible to make the data stored in Postgres persist between runs. To do
so, change our ``postgres`` container as follows:

.. code-block:: yaml

    containers:
      postgres:
        setup:
        - !Ubuntu xenial
        - !Install [postgresql]
        - !EnsureDir /data
        - !EnsureDir /work/.db/data ❶
        environ:
          PGDATA: /data
          PG_PORT: 5433
          PG_DB: test
          PG_USER: vagga
          PG_PASSWORD: vagga
          PG_BIN: /usr/lib/postgresql/9.5/bin
        volumes:
          /data: !BindRW /work/.db/data ❷

* ❶ -- we will persist postgres data in ``.db/data``, so ensure it exists
* ❷ -- bind ``/data`` to our persistent directory instead of "!Tmpfs"

And also change the ``run-postgres`` command:

.. code-block:: yaml

  commands:
    run-postgres: !Supervise
    description: Start the django development server using Postgres database
    children:
      # ...
      db: !Command
        container: postgres
        run: |
            chown postgres:postgres $PGDATA;
            if [ -z $(ls -A $PGDATA) ]; then ❶
              su postgres -c "$PG_BIN/pg_ctl initdb";
              su postgres -c "echo 'host all all all trust' >> $PGDATA/pg_hba.conf"
              su postgres -c "$PG_BIN/pg_ctl -w -o '-F --port=$PG_PORT -k /tmp' start";
              su postgres -c "$PG_BIN/psql -h 127.0.0.1 -p $PG_PORT -c \"CREATE USER $PG_USER WITH PASSWORD '$PG_PASSWORD';\""
              su postgres -c "$PG_BIN/createdb -h 127.0.0.1 -p $PG_PORT $PG_DB -O $PG_USER";
            else ❷
              su postgres -c "$PG_BIN/pg_ctl -w -o '-F --port=$PG_PORT -k /tmp' start";
            fi
            rm /work/.dbcreation # Release lock
            sleep infinity

* ❶ -- check if there is already a database created
* ❷ -- otherwise just start the database

These changes will persist the database files inside ``.db/data`` on the project
directory. We will not have any permission on that directory, so we would not be
able to list its contents nor delete it, unless we are root.

Note that if we delete the ``.db/data`` directory, we will get the error::

    Can't mount bind "/work/.db/data" to "/vagga/root/data": No such file or directory

To solve that, simply recreate ``.db/data``.
