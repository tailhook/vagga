================
Django on Alpine
================

This example will show how to create a simple Django application in an Alpine container.

- `Creating the project structure`_
- `Freezing dependencies`_
- `Let's add a dependency`_
- `Making a blog: The hello world of web frameworks`_
- `Trying out memcached`_

Creating the project structure
------------------------------

In order to create the initial project structure, we need a container with Django
installed, so create a ``vagga.yaml`` and add the following to it:

.. code-block:: yaml

  containers:
    django:
      setup:
      - !Alpine v3.3
      - !Py3Install ['Django >=1.9,<1.10']

and then run:

.. code-block::

  $ vagga _run django django-admin startproject MyProject .

This will create a project named ``MyProject`` in the current directory.

Freezing dependencies
---------------------

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
        - 'Django >=1.9,<1.10' # you can change if there is a higher version
      - !Sh pip freeze > requirements.txt
    django:
      setup:
      - !Alpine v3.3
      - !Py3Requirements requirements.txt

Now, build the ``app-freezer`` container:

.. code-block::

  $ vagga _build app-freezer

You will notice the new ``requirements.txt`` file holding a content similar to:

.. code-block::

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

and then run:

.. code-block::

  $ vagga run

If everything went right, visiting ``localhost:8000`` will display Django's welcome
page saying 'It worked!'.

Let's add a dependency
----------------------

By default, Django is configured to use sqlite as its database, but we want to
use a database url from an environment variable, since it's more flexible, so we
will add ``dj-database-url`` to our project.

First, add ``dj-database-url`` to our ``app-freezer`` container:

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
    # ...

Second, rebuild the ``app-freezer`` container to update ``requirements.txt``

.. code-block::

  $ vagga _build app-freezer

Third, set the environment variable

.. code-block:: yaml

  containers:
    #...
    django:
      environ:
        DATABASE_URL: sqlite:///db.sqlite3 # will point to /work/db.sqlite3
      setup:
      - !Alpine v3.3
      - !Py3Requirements requirements.txt
    # ...

Now let's change our project's settings by editing ``MyProject/settings.py``:

.. code-block:: python

  import os
  import dj_database_url
  # ...
  databases = {
      'default': dj_database_url.config()
  }

To see if it worked, let's run the migrations from the default Django apps and
create a superuser:

.. code-block::

  $ vagga _run django python3 manage.py migrate
  $ vagga _run django python3 manage.py createsuperuser

After creating the superuser, run ``vagga run``, visit ``localhost:8000/admin``
and log into our project.

Making a blog: The hello world of web frameworks
------------------------------------------------

Before going any further, let's add something to our project, like a blogging
platform!

First, start an app called 'blog':

.. code-block::

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
  from django.conf import settings
  from django.db import models


  class Article(models.Model):
      title = models.CharField(max_length=100)
      body = models.TextField()
      date = models.DateField()
      created_at = models.DateTimeField(auto_now_add=True)
      updated_at = models.DateTimeField(auto_now=True)

      author = models.ForeignKey(settings.AUTH_USER_MODEL)

      class Meta:
          ordering = ['-date']

Create the admin for our model:

.. code-block:: python

  # blog/admin.py
  from django.contrib import admin
  from .models import Article


  @admin.register(Article)
  class ArticleAdmin(admin.ModelAdmin):
      fields = ('title', 'date', 'body')
      list_display = ('title', 'date', 'created_at', 'updated_at', 'author')

      def save_model(self, request, obj, form, change):
          if not change:
              obj.author = request.user
          obj.save()

Create and run the migration:

.. code-block::

  $ vagga _run django python3 manage.py makemigrations
  $ vagga _run django python3 manage.py migrate

Run the project:

.. code-block::

  $ vagga run

And visit ``localhost:8000/admin`` to see our new model in action.

Now create a view:

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

  <!-- blog/templates/blog/article_list.html -->
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

  <!-- blog/templates/blog/article_detail.html -->
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

  # MyProject/urls.py
  from django.conf.urls import url, include
  from django.contrib import admin

  urlpatterns = [
      url(r'^', include('blog.urls', namespace='blog')),
      url(r'^admin/', admin.site.urls),
  ]

Run our project and visit ``localhost:8000``. Try adding some data through the
admin to see the result.

Trying out memcached
--------------------

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

And rebuild the container:

.. code-block::

  $ vagga _build app-freezer

Add the ``pylibmc`` runtime dependencies to our ``django`` container:

.. code-block:: yaml

  containers:
    # ...
    django:
      environ:
        DATABASE_URL: sqlite:///db.sqlite3
      setup:
      - !Alpine v3.3
      - !Install
        - libmemcached
        - zlib
        - libsasl
      - !Py3Requirements requirements.txt

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

  urlpatterns = [
      url(r'^$', views.ArticleList.as_view(), name='article_list'),
      url(r'^(?P<pk>\d+?)$', cache_page(60 * 15)(views.ArticleDetail.as_view()), name='article_detail'),
  ]

And run our project, visit any article detail page, hit ``Ctrl+r`` to avoid
browser cache and watch the memcached output on the terminal.
