================
Django on Alpine
================

This example will show how to create a simple Django application while showing
some more advanced uses of vagga with Django.

* `Creating the project structure`_
* `Freezing dependencies`_
* `Let's add a dependency`_
* `Adding some code`_
* `Trying out memcached`_
* `Why not Postgres?`_

Creating the project structure
==============================

In order to create the initial project structure, we need a container with Django
installed, so create a ``vagga.yaml`` file and add the following to it:

.. code-block:: yaml

    containers:
      django:
        setup:
        - !Alpine v3.3
        - !Py3Install ['Django >=1.9,<1.10'] # you can change if there is a higher version

and then run::

    $ vagga _run django django-admin startproject MyProject .

This will create a project named ``MyProject`` in the current directory.

Freezing dependencies
=====================

It is a common practice for python projects to have a ``requirements.txt`` file
that will hold the exact versions of the project dependencies. This way, any
developer working on the project will have the same dependencies.

In order to generate the ``requirements.txt`` file, we will create another
container to do this task. Start by modifying our ``vagga.yaml`` as follows:

.. code-block:: yaml

    containers:
      app-freezer:
        setup:
        - !Alpine v3.3
        - !Py3Install
          - pip
          - 'Django >=1.9,<1.10'
        - !Sh pip freeze > requirements.txt
      django:
        setup:
        - !Alpine v3.3
        - !Py3Requirements requirements.txt

Now, build the ``app-freezer`` container::

    $ vagga _build app-freezer

You will notice the new ``requirements.txt`` file holding a content similar to::

    Django==1.9.2

You may have noticed we used ``'Django >=1.9,<1.10'`` instead of just ``Django``.
It is a good practice to always specify the major and minor versions of a dependency.
This prevents an update to an incompatible version of a library breaking you project.

Now let's run our django project. Change our ``vagga.yaml`` as follows:

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

If everything went right, visiting ``localhost:8000`` will display Django's welcome
page saying 'It worked!'.

Let's add a dependency
======================

By default, Django is configured to use sqlite as its database, but we want to
use a database url from an environment variable, since it's more flexible, so we
will add ``dj-database-url`` to our project.

Add ``dj-database-url`` to our ``app-freezer`` container:

.. code-block:: yaml

    containers:
      app-freezer:
        setup:
        - !Alpine v3.3
        - !Py3Install
          - pip
          - 'Django >=1.9,<1.10'
          - 'dj-database-url >=0.4,<0.5'
        - !Sh pip freeze > requirements.txt

Rebuild the ``app-freezer`` container to update ``requirements.txt``::

    $ vagga _build app-freezer

Set the environment variable

.. code-block:: yaml

    containers:
      #...
      django:
        environ:
          DATABASE_URL: sqlite:///db.sqlite3 # will point to /work/db.sqlite3
        setup:
        - !Alpine v3.3
        - !Py3Requirements requirements.txt

Now let's change our project's settings by editing ``MyProject/settings.py``:

.. code-block:: python

    # MyProject/settings.py
    import os
    import dj_database_url
    # ...
    DATABASES = {
        'default': dj_database_url.config()
    }

To see if it worked, let's run the migrations from the default Django apps and
create a superuser::

    $ vagga _run django python3 manage.py migrate
    $ vagga _run django python3 manage.py createsuperuser

After creating the superuser, run our project::

    $ vagga run

visit ``localhost:8000/admin`` and log into our project.

Adding some code
================

Before going any further, let's add something to our project, like a blogging
platform.

First, start an app called 'blog'::

    $ vagga _run django python3 manage.py startapp blog

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

    $ vagga _run django python3 manage.py makemigrations
    $ vagga _run django python3 manage.py migrate

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
      <p>{{ article.date }}</p>
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

and visit ``localhost:8000``. Try adding some data through the admin to see the
result.

Trying out memcached
====================

Many applications use `memcached <http://memcached.org/>`_ to speed up things, so
let's try it out.

Add ``pylibmc`` and ``django-cache-url`` to our ``app-freezer``, as well as the
build dependencies of ``pylibmc``:

.. code-block:: yaml

    containers:
      app-freezer:
        setup:
        - !Alpine v3.3
        - !BuildDeps
          - libmemcached-dev
          - zlib-dev
        - !Py3Install
          - pip
          - 'Django >=1.9,<1.10'
          - 'dj-database-url >=0.4,<0.5'
          - 'pylibmc >=1.5,<1.6'
          - 'django-cache-url >=1.0,<1.1'
        - !Sh pip freeze > requirements.txt

And rebuild the container::

    $ vagga _build app-freezer

Add the ``pylibmc`` runtime dependencies to our ``django`` container:

.. code-block:: yaml

    containers:
      # ...
      django:
        setup:
        - !Alpine v3.3
        - !Install
          - libmemcached
          - zlib
          - libsasl
        - !Py3Requirements requirements.txt
        environ:
          DATABASE_URL: sqlite:///db.sqlite3

Crate a new container called ``memcached``:

.. code-block:: yaml

    containers:
      # ...
      memcached:
        setup:
        - !Alpine v3.3
        - !Install [memcached]

Create the command to run with caching:

.. code-block:: yaml

    # ...
    commands:
      # ...
      run-cached: !Supervise
        description: Start the django development server alongside memcached
        children:
          cache: !Command
            container: memcached
            run: memcached -u memcached -vv # verbose to let us see the cache working
          app: !Command
            container: django
            environ:
              CACHE_URL: memcached://127.0.0.1:11211
            run: python3 manage.py runserver

Change our ``MyProject/settings.py`` as follows:

.. code-block:: python

    import os
    import dj_database_url
    import django_cache_url
    # ...
    CACHES = {
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
        - !Alpine v3.3
        - !BuildDeps
          - libmemcached-dev
          - zlib-dev
          - postgresql-dev
        - !Py3Install
          - pip
          - 'Django >=1.9,<1.10'
          - 'dj-database-url >=0.4,<0.5'
          - 'pylibmc >=1.5,<1.6'
          - 'django-cache-url >=1.0,<1.1'
          - 'psycopg2 >=2.6,<2.7'
        - !Sh pip freeze > requirements.txt

Rebuild the container::

    $ vagga _build app-freezer

Add the runtime dependencies of ``psycopg2``:

.. code-block:: yaml

    containers:
      django:
        setup:
        - !Alpine v3.3
        - !Install
          - libmemcached
          - zlib
          - libsasl
          - libpq
        - !Py3Requirements requirements.txt
        environ:
          DATABASE_URL: sqlite:///db.sqlite3

Before running our project, we need a way to automatically create our superuser.
We can crate a migration to do this. First, create an app called ``common``::

    $ vagga _run django python3 manage.py startapp common

Add it to ``INSTALLED_APP``:

.. code-block:: python

    INSTALLED_APPS = [
        # ...
        'common',
        'blog',
    ]

Create the migration for adding the admin user::

    $ vagga _run django python3 manage.py makemigrations -n create_admin_user --empty common

Change the migration to add our admin user:

.. code-block:: python

    # common/migrations/0001_create_admin_user.py
    from django.db import migrations
    from django.contrib.auth.hashers import make_password


    def create_admin_user(apps, schema_editor):
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
            migrations.RunPython(create_admin_user)
        ]

Add the database container:

.. code-block:: yaml

    containers:
      #..
      postgres:
        setup:
        - !Ubuntu trusty
        - !Install [postgresql]
        - !EnsureDir /data
        environ:
          PGDATA: /data
          PG_PORT: 5433
          PG_DB: test
          PG_USER: vagga
          PG_PASSWORD: vagga
          PG_BIN: /usr/lib/postgresql/9.3/bin
        volumes:
          /data: !Tmpfs
            size: 100M
            mode: 0o700

And the command to run with Postgres:

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
