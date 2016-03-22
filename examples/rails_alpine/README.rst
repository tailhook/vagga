========================
Building a Rails project
========================

This example will show how to create a simple Rails project using vagga.

* `Creating the project structure`_
* `Configuring the database from environment`_
* `Adding some code`_
* `Caching with memcached`_
* `We should try Postgres too`_

Creating the project structure
==============================

First, let's create a directory for our new project::

    $ mkdir -p ~/projects/vagga-rails-tutorial && cd ~/projects/vagga-rails-tutorial

Now we need to create our project's structure, so let's create a new container
and tell it to do so.

Create the ``vagga.yaml`` file and add the following to it:

.. code-block:: yaml

    containers:
      rails:
        setup:
        - !Alpine v3.3
        - !Install ❶
          - libxml2
          - libxslt
          - zlib
        - !BuildDeps ❶
          - libxml2-dev
          - libxslt-dev
          - zlib-dev
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
* ❹ -- The ``rails new`` command, which we are going to use shortly, will
  complain if we do not have a ``$HOME``. After our project is created, we won't
  need it anymore.

.. _nokogiri: http://www.nokogiri.org
.. _nokogiri docs: http://www.nokogiri.org/tutorials/installing_nokogiri.html

And now run::

    $ vagga _run rails rails new . --skip-bundle

This will create a new rails project in the current directory. The ``--skip-bundle``
flag tells ``rails new`` to not run ``bundle install``, but don't worry, vagga
will take care of it for us.

Now that we have our rails project, let's change our container fetch dependencies
from ``Gemfile``:

.. code-block:: yaml

    containers:
      rails:
        setup:
        - !Alpine v3.3
        - !Install
          - libxml2
          - libxslt
          - zlib
          - sqlite-libs ❶
          - nodejs ❶
        - !BuildDeps
          - libxml2-dev
          - libxslt-dev
          - zlib-dev
          - sqlite-dev ❶
        - !Env
          NOKOGIRI_USE_SYSTEM_LIBRARIES: 1
        - !GemBundle ❷

* ❶ -- we need ``sqlite`` for the development database and ``nodejs`` for the
  asset pipeline (specifically, the ``uglifier`` gem).
* ❷ -- install dependencies from ``Gemfile`` using ``bundle install``.

Before we test our project, let's add two gems into the ``Gemfile``:

.. code-block:: ruby

    # Gemfile
    # ...
    gem 'bigdecimal'
    gem 'tzinfo-data'
    # ...

Without these two gems, you may run into import errors.

To test if everything is Ok, let's create a command to run our project:

.. code-block:: yaml

    commands:
      run: !Command
        container: rails
        description: start rails development server
        run: rails server

Run the project::

    $ vagga run

Now visit ``localhost:3000`` to see rails default page.

Configuring the database from environment
=========================================

By default, the ``rails new`` command will setup sqlite as the project database
and store the configuration in ``config/databse.yml``. However, we will use an
environment variable to tell rails where is our database. To do so, first delete
the rails database file::

    $ rm config/database.yml

And set the enviroment variable in our ``vagga.yaml``:

.. code-block:: yaml

    containers:
      rails:
        setup:
          # ...
        environ:
          DATABASE_URL: sqlite3:db/development.sqlite3

This will tell rails to use the same file that was configured in ``database.yml``.

Now if we run our project, everything should be the same.

.. note:: You may need to remove "tmp/pids/server.pid", otherwise, rails will
  complain that the server is already running.

Adding some code
================

Before going any further, let's add some code to our project::

    $ vagga _run rails rails g scaffold article title:string:index body:text

Rails scaffolding will generate everything we need, and now we just have to run
the migration::

    $ vagga _run rails rake db:migrate

Now we just have to tell rails to use our articles index page as the root of our
project. Edit ``config/routes.rb`` as follows:

.. code-block:: ruby

    # config/routes.rb

    Rails.application.routes.draw do
      root 'articles#index'
      resources :articles
      # ...
    end

If you run the project now it will show the articles list page.

Caching with memcached
======================

Many projects use `memcached <http://memcached.org/>`_ to speed up things, so
let's try it out.

First, add ``dalli``, a pure ruby memcached client, to our ``Gemfile``:

.. code-block:: ruby

    # ...
    gem 'dalli'

Then, open ``config/environments/production.rb``, find the line containing
``# config.cache_store`` and edit it as follows:

.. code-block:: ruby

    # config/environments/production.rb
    Rails.application.configure do
      # ...
      config.cache_store = :mem_cache_store, ENV['CACHE_URL']
      # ...
    end

.. note:: You should only setup caching in production environment.

Create a container for memcached:

.. code-block:: yaml

    containers:
      # ...
      memcached:
        setup:
        - !Alpine v3.3
        - !Install [memcached]

Create the command to run with caching:

.. code-block:: yaml

    commands:
      # ...
      run-cached: !Supervise
        description: Start the rails development server alongside memcached
        children:
          cache: !Command
            container: memcached
            run: memcached -u memcached -vv ❶
          app: !Command
            container: rails
            environ:
              CACHE_URL: memcached://127.0.0.1:11211 ❷
              RAILS_ENV: production ❸
              SECRET_KEY_BASE: my_secret_key ❹
              RAILS_SERVE_STATIC_FILES: 1 ❺
            run: |
                rake assets:precompile ❻
                rails server

* ❶ -- run memcached as verbose so we see can see the cache working
* ❷ -- set the cache url
* ❸ -- tell rails to run in production environment
* ❹ -- production environment requires a secret key
* ❺ -- tell rails to serve static files on production environment
* ❻ -- precompile assets

Now let's change some of our views to use caching:

.. code-block:: html+erb

    <!-- app/views/articles/show.html.erb -->
    <%# ... %>
    <% cache @article do %>
      <p>
        <strong>Title:</strong>
        <%= @article.title %>
      </p>

      <p>
        <strong>Body:</strong>
        <%= @article.body %>
      </p>
    <% end %>
    <%# ... %>

.. code-block:: html+erb

    <!-- app/views/articles/index.html.erb -->
    <%# ... %>
    <table>
      <%# ... %>
      <tbody>
        <% @articles.each do |article| %>
          <% cache article do %>
            <tr>
              <td><%= article.title %></td>
              <td><%= article.body %></td>
              <td><%= link_to 'Show', article %></td>
              <td><%= link_to 'Edit', edit_article_path(article) %></td>
              <td><%= link_to 'Destroy', article, method: :delete, data: { confirm: 'Are you sure?' } %></td>
            </tr>
          <% end %>
        <% end %>
      </tbody>
    </table>
    <%# ... %>

Run the project with caching::

    $ vagga run-cached

Try adding some records. Keep an eye on the console to see rails talking to
memcached.

We should try Postgres too
==========================

We can test our project against a Postgres database, which is probably what we
will use in production.

First, add gem ``pg`` to our ``Gemfile``

.. code-block:: ruby

    gem 'pg'

Then add the system dependencies for gem ``pg``

.. code-block:: yaml

    containers:
      rails:
        setup:
        - !Alpine v3.3
        - !Install
          - libxml2
          - libxslt
          - zlib
          - sqlite-libs
          - libpq ❶
          - nodejs
        - !BuildDeps
          - libxml2-dev
          - libxslt-dev
          - zlib-dev
          - sqlite-dev
          - postgresql-dev ❷
        - !Env
          NOKOGIRI_USE_SYSTEM_LIBRARIES: 1
        - !GemBundle
        environ:
          DATABASE_URL: sqlite3:db/development.sqlite3

* ❶ -- runtime dependency
* ❷ -- build dependency

Create the database container

.. code-block:: yaml

    containers:
      # ...
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

And then add the command to run with Postgres:

.. code-block:: yaml

    commands:
      # ...
      run-postgres: !Supervise
        description: Start the rails development server using Postgres database
        children:
          app: !Command
            container: rails
            environ:
              DATABASE_URL: postgresql://vagga:vagga@127.0.0.1:5433/test
            run: |
                touch /work/.dbcreation # Create lock file
                while [ -f /work/.dbcreation ]; do sleep 0.2; done # Acquire lock
                rake db:migrate
                rails server
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
