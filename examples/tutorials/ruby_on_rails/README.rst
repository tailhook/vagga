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
        - !Ubuntu xenial
        - !Install ❶
          - zlib1g
        - !BuildDeps ❶
          - zlib1g-dev
        - !GemInstall [rails:5.0] ❷
        environ:
          HOME: /tmp ❸

* ❶ -- ``rails`` depends on `nokogiri`_, which depends on zlib.
* ❷ -- tell ``gem`` to install ``rails``.
* ❸ -- The ``rails new`` command, which we are going to use shortly, will
  complain if we do not have a ``$HOME``. After our project is created, we won't
  need it anymore.

.. _nokogiri: http://www.nokogiri.org

We explicitly installed rails version 5.0. You can change to a newer version if
it is available (5.1, for example) but your project may be slightly different.

And now run::

    $ vagga _run rails rails new . --skip-bundle

This will create a new rails project in the current directory. The ``--skip-bundle``
flag tells ``rails new`` to not run ``bundle install``, but don't worry, vagga
will take care of it for us.

Now that we have our rails project, let's change our container fetch dependencies
from ``Gemfile``:

.. code-block:: yaml

    containers:
      base:
        setup:
        - !Ubuntu xenial
        - !UbuntuUniverse
        - !Install
          - zlib1g
          - libsqlite3-0 ❶
          - nodejs ❶
        - !BuildDeps
          - zlib1g-dev
          - libsqlite3-dev
        - !GemInstall
          - ffi
          - nokogiri
          - sqlite3
      rails:
        setup:
        - !Container base
        - !GemBundle ❷

* ❶ -- we need ``sqlite`` for the development database and ``nodejs`` for the
  asset pipeline (specifically, the ``uglifier`` gem).
* ❷ -- install dependencies from ``Gemfile`` using ``bundle install``.

We are using two containers here, ``base`` and ``rails``, for a good reason:
some gems require building modules that can take some time to compile, so
building them on the ``base`` container will avoid having to build them every
time we need to rebuild our main container.

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

.. note:: You may need to remove "tmp/pids/server.pid" in subsequent runs,
  otherwise, rails will complain that the server is already running.

Configuring the database from environment
=========================================

By default, the ``rails new`` command will setup sqlite as the project database
and store the configuration in ``config/databse.yml``. However, we will use an
environment variable to tell rails where to find our database. To do so, delete
the rails database file::

    $ rm config/database.yml

And then set the enviroment variable in our ``vagga.yaml``:

.. code-block:: yaml

    containers:
      rails:
        setup:
          # ...
        environ:
          DATABASE_URL: sqlite3:db/development.sqlite3

This will tell rails to use the same file that was configured in ``database.yml``.

Now if we run our project, everything should be the same.

Adding some code
================

Before going any further, let's add some code to our project::

    $ vagga _run rails rails g scaffold article title:string:index body:text

Rails scaffolding will generate everything we need, we just have to run the
migrations::

    $ vagga _run rails rake db:migrate

Now we need to tell rails to use our articles index page as the root of our
project. Change ``config/routes.rb`` as follows:

.. code-block:: ruby

    # config/routes.rb

    Rails.application.routes.draw do
      root 'articles#index'
      resources :articles
      # ...
    end

Run the project now::

    $ vagga run

You should see the articles list page rails generated for us.

Caching with memcached
======================

Many projects use `memcached <http://memcached.org/>`_ to speed up things, so
let's try it out.

First, add ``dalli``, a pure ruby memcached client, to our ``Gemfile``:

.. code-block:: ruby

    gem 'dalli'

Then, open ``config/environments/development.rb``, find the line that says
``config.cache_store = :memory_store`` and change it as follows:

.. code-block:: ruby

    # config/environments/production.rb
    # ...
    # config.cache_store = :memory_store
    if ENV['MEMCACHED_URL']
      config.cache_store = :mem_cache_store, ENV['MEMCACHED_URL']
    else
      config.cache_store = :memory_store
    end
    # ...

Create a container for memcached:

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
        description: Start the rails development server alongside memcached
        children:
          cache: !Command
            container: memcached
            run: memcached -u memcached -vv ❶
          app: !Command
            container: rails
            environ:
              MEMCACHED_URL: memcached://127.0.0.1:11211 ❷
            run: |
                if [ ! -f 'tmp/caching-dev.txt' ]; then
                  touch tmp/caching-dev.txt ❸
                fi
                rails server

* ❶ -- run memcached as verbose so we see can see the cache working
* ❷ -- set the cache url
* ❸ -- creating this file will tell rails to activate cache in development

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
      base:
        setup:
        - !Ubuntu xenial
        - !UbuntuUniverse
        - !Install
          - zlib1g
          - libsqlite3-0
          - nodejs
          - libpq5 ❶
        - !BuildDeps
          - zlib1g-dev
          - libsqlite3-dev
          - libpq-dev ❷
        - !GemInstall
          - ffi
          - nokogiri
          - sqlite3
          - pg
      rails:
        setup:
        - !Container base
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

We can also add some default records to the database, so we don't have to add
them everytime we run our project. To do so, add the following to ``db/seeds.rb``:

.. code-block:: ruby

    # db/seeds.rb
    Article.create([
      { title: 'Article 1', body: 'Lorem ipsum dolor sit amet' },
      { title: 'Article 2', body: 'Lorem ipsum dolor sit amet' },
      { title: 'Article 3', body: 'Lorem ipsum dolor sit amet' }
    ])

Now change the ``run-postgres`` command to seed the database:

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
                rake db:seed ❶
                rails server
          db: !Command
            # ...

* ❶ -- populate the database.

Now , everytime we run ``run-postgres``, we will have our database populated.
