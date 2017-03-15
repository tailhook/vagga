==========================
Building a Laravel project
==========================

This tutorial will show how to create a simple Laravel_ project using vagga.

* `Creating the project`_
* `Setup the database`_
* `Adding some code`_
* `Setup Redis`_
* `Deploying to a shared server`_

.. _Laravel: https://laravel.com/


Creating the project
====================

In order to create the initial project structure, we will need a container with
the Laravel installer. First, let's create a directory for our project::

    $ mkdir -p ~/projects/vagga-laravel-tutorial && cd ~/projects/vagga-laravel-tutorial

Create the ``vagga.yaml`` file and add the following to it:

.. code-block:: yaml

    containers:
      app:
        setup:
        - !Alpine v3.5
        - !Repo community
        - !Install
          - ca-certificates
          - php7
          - php7-openssl
          - php7-mbstring
          - php7-phar
          - php7-json
        - !ComposerConfig
          install-runtime: false
          runtime-exe: /usr/bin/php7
          keep-composer: true
        - !ComposerInstall
        environ:
          HOME: /tmp

Here we are building a container from Alpine v3.5 and telling it to install PHP7
and everything needed to run Composer. Now let's create our new project::

    $ vagga _run app composer create-project \
        --prefer-dist --no-install --no-scripts \
        laravel/laravel src 5.4.*
    $ mv src/* src/.* .
    $ rmdir src

The first command is quite big! It tells composer to create a new project from
``laravel/laravel`` version 5.4 and place it into the ``src`` directory. The three
flags tell composer to:

* ``--prefer-dist`` install packages from distribution source when available;
* ``--no-install`` do not run ``composer install`` after creating the project;
* ``--no-scripts`` do not run scripts defined in the root package.

We want our project's files to be in the current directory (the one containing
``vagga.yaml``) but Composer only accepts an empty directory, so we tell it to
create the project into ``src``, move its contents into the current directory
and remove ``src``.

Now that we have our project created, change our container as follows:

.. code-block:: yaml

    containers:
      app-base:
        setup:
        - !Alpine v3.5
        - !Repo community
        - !Install
          - ca-certificates
          - php7
          - php7-openssl
          - php7-pdo_mysql
          - php7-mbstring
          - php7-xml
          - php7-session
          - php7-dom
          - php7-phar
          - php7-json
          - php7-posix
          - php7-ctype
        - !Sh ln -s /usr/bin/php7 /usr/bin/php
      app:
        environ: &env
          APP_ENV: development
          APP_DEBUG: true
          APP_KEY: YourRandomGeneratedEncryptionKey
        setup:
        - !Container app-base
        - !Env { <<: *env }
        - !ComposerConfig
          install-runtime: false
          runtime-exe: /usr/bin/php7
          keep-composer: true
        - !EnsureDir /work/vendor
        - !EnsureDir /usr/local/lib/composer/vendor
        - !Sh mount --bind,ro /usr/local/lib/composer/vendor /work/vendor
        - !ComposerDependencies
        - !Sh umount /work/vendor
        volumes:
          /work/vendor: !BindRO /vagga/root/usr/local/lib/composer/vendor

This might look complex, but let's break it down:

.. code-block:: yaml

    app-base:
      setup:
      - !Alpine v3.5
      - !Repo community
      - !Install
        - ca-certificates
        - php7
        - php7-openssl
        - php7-pdo_mysql
        - php7-mbstring
        - php7-xml
        - php7-session
        - php7-dom
        - php7-phar
        - php7-json
        - php7-posix
        - php7-ctype
      - !Sh ln -s /usr/bin/php7 /usr/bin/php

The container for our application is based on Alpine linux v3.5 and we will use
PHP7, so we need to enable the "community" repository from Alpine and install
php7 and the modules needed for both Laravel and Composer.

We also link the php7 executable into ``/usr/bin/php`` to make it available as
just ``php``.

This container will be used as the base for another container in order to speed
up builds.

.. code-block:: yaml

    environ: &env
      APP_ENV: development
      APP_DEBUG: true
      APP_KEY: YourRandomGeneratedEncryptionKey

Here we are configuring our application. Laravel comes out of the box with its
configuration done through environment variables, so we are setting these to
what we need to a development environment. The default project template uses
`dotenv`_ to load configuration into environment automatically from a ``.env``
file, but we won't use that. Instead, we tell vagga to set the environment for us.

We are also setting and yaml anchor (``&env``) so we can reference it later.

.. code-block:: yaml

    setup:
    - !Container app-base
    - !Env { <<: *env }

We are extending the ``app-base`` container and referencing the yaml anchor we
defined earlier to make the environment available during build.

.. code-block:: yaml

    - !ComposerConfig
      install-runtime: false
      runtime-exe: /usr/bin/php7
      keep-composer: true

Since we installed php by ourselves, we tell vagga to use version we installed
instead of the default version from Alpine.

.. code-block:: yaml

    - !EnsureDir /work/vendor
    - !EnsureDir /usr/local/lib/composer/vendor
    - !Sh mount --bind,ro /usr/local/lib/composer/vendor /work/vendor
    - !ComposerDependencies
    - !Sh umount /work/vendor

Applications using Composer usually expect the ``vendor`` directory to be
available at the project root, but vagga install composer dependencies under
``/usr/local/lib/composer``. To make it available to our application, we mount
that directory into ``/work/vendor`` and ``umount`` after build.

To test if everything is ok, let's add a command to run our project:

.. code-block:: yaml

    containers:
      # ...
    commands:
      run: !Command
        container: app
        description: run the laravel development server
        run: |
            php artisan cache:clear ❶
            php artisan config:clear ❶
            php artisan serve

* ❶ -- clear application cache to prevent previous runs from intefering on
  subsequent runs.

Now run our project::

    $ vagga run

And visit ``localhost:8000``. If everithing is OK, you will see Laravel default
page saying "Laravel 5".

.. _dotenv: https://github.com/vlucas/phpdotenv

Setup the database
==================

Every PHP project needs a database, and ours is not different, so let's create a
container for our database:

.. code-block:: yaml

    containers:
      # ...
      mysql:
        setup:
        - !Ubuntu xenial
        - !UbuntuUniverse
        - !Sh |
            addgroup --system --gid 200 mysql ❶
            adduser --uid 200 --system --home /data --no-create-home \
                --shell /bin/bash --group --gecos "MySQL user" \
                mysql
        - !Install
          - mysql-server-5.7
          - mysql-client-5.7
        - !Remove /var/lib/mysql
        - !EnsureDir /data
        environ: &db_config ❷
          DB_DATABASE: vagga
          DB_USERNAME: vagga
          DB_PASSWORD: vagga
          DB_HOST: 127.0.0.1
          DB_PORT: 3307
          DB_DATA_DIR: /data
        volumes:
          /data: !Persistent
            name: mysql
            owner-uid: 200
            owner-gid: 200
            init-command: _mysql-init ❸
          /run: !Tmpfs
            subdirs:
              mysqld: { mode: 0o777 }

* ❶ -- Use fixed user id and group id for mysql
* ❷ -- Put an anchor at the database environment so we can reference it later
* ❸ -- Vagga command to initialize the volume

.. note:: The database will be persisted in ``.vagga/.volumes/mysql``.

Add the command to initialize the database:

.. code-block:: yaml

    commands:
      # ...
      _mysql-init: !Command
        description: Init MySQL data volume
        container: mysql
        user-id: 200
        group-id: 200
        run: |
          set -ex

          mysqld --initialize-insecure --datadir=$DB_DATA_DIR \
            --log-error=log

          mysqld --datadir=$DB_DATA_DIR --skip-networking --log-error=log &

          while [ ! -S /run/mysqld/mysqld.sock ]; do sleep 0.2; done

          mysqladmin -u root create $DB_DATABASE
          mysql -u root -e "CREATE USER '$DB_USERNAME'@'localhost' IDENTIFIED BY '$DB_PASSWORD';"
          mysql -u root -e "GRANT ALL PRIVILEGES ON $DB_DATABASE.* TO '$DB_USERNAME'@'localhost';"
          mysql -u root -e "FLUSH PRIVILEGES;"

          mysqladmin -u root shutdown

Add a the php mysql module to our container:

.. code-block:: yaml

    containers:
      app-base:
        - !Alpine v3.5
        - !Repo community
        - !Install
          - ca-certificates
          - php7
          # ...
          - php7-pdo_mysql # mysql module
        # ...

Now change our ``run`` command to start the database alongside our project:

.. code-block:: yaml

    commands:
      run: !Supervise
        description: run the laravel development server
        children:
          app: !Command
            container: app
            environ: *db_config ❶
            run: |
                php artisan cache:clear
                php artisan config:clear
                php artisan serve
          db: !Command
            container: mysql
            user-id: 200
            group-id: 200
            run: |
              exec mysqld --datadir=$DB_DATA_DIR \
                --bind-address=$DB_HOST --port=$DB_PORT \
                --log-error=log --gdb

* ❶ -- Reference the database environment

And run our project::

    $ vagga run

Inspecting the database
=======================

Now that we have a working database, we can inspect it using a small php utility
called `adminer`_. Let's create a container for it:

.. code-block:: yaml

    containers:
      # ...
      adminer:
        setup:
        - !Alpine v3.5
        - !Repo community
        - !Install
          - php7
          - php7-pdo_mysql
          - php7-session
        - !EnsureDir /opt/adminer
        - !EnsureDir /opt/adminer/plugins
        - !Download
          url: https://www.adminer.org/static/download/4.2.5/adminer-4.2.5-mysql.php ❶
          path: /opt/adminer/adminer.php
        - !Download
          url: https://raw.github.com/vrana/adminer/master/designs/nette/adminer.css ❷
          path: /opt/adminer/adminer.css
        - !Download
          url: https://raw.github.com/vrana/adminer/master/plugins/plugin.php ❸
          path: /opt/adminer/plugins/plugin.php
        - !Download
          url: https://raw.github.com/vrana/adminer/master/plugins/login-servers.php ❹
          path: /opt/adminer/plugins/login-servers.php
        - !Text
          /opt/adminer/index.php: |
              <?php ❺
              function adminer_object() {
                  include_once "./plugins/plugin.php";
                  foreach (glob("plugins/*.php") as $filename) { include_once "./$filename"; }
                  $plugins = [new AdminerLoginServers(['127.0.0.1:3307' => 'Dev DB'])];
                  return new AdminerPlugin($plugins);
              }
              include "./adminer.php";

* ❶ -- download the adminer script.
* ❷ -- use a better style (optional).
* ❸ -- adminer plugin support
* ❹ -- login-servers plugin to avoid typing server address and port
* ❺ -- setup adminer

The container above will install PHP7 along with the mysql and session modules,
then it will download adminer itself, the optional style, the plugin support and
the "login-servers" plugin. This plugin will allow us to select the database we
are connecting to from a list instead of filling in the host and port. The last
part of the container setup configures adminer with our database.

Now change our ``run`` command to start the adminer container:

.. code-block:: yaml

    commands:
      run: !Supervise
        description: run the laravel development server
        children:
          app: !Command
            # ...
          db: !Command
            # ...
          adminer: !Command
            container: adminer
            run: php7 -S 127.0.0.1:8001 -t /opt/adminer

This command will start the php embedded server with its root pointing to the
directory we setup for Adminer.

To access adminer, visit ``localhost:8001`` and fill the username and password
fields with "vagga".

.. _`adminer`: https://www.adminer.org

Adding some code
================

Now that we have our project working and our database is ready, let's add some.

Let's add a shortcut command for running artisan

.. code-block:: yaml

    commands:
      # ...
      artisan: !Command
        description: Shortcut for running artisan cli
        container: app
        run: [php, artisan]

Now, we need a layout. Fortunately, Laravel can give us one, we just have to
scaffold authentication::

    $ vagga artisan make:auth

This will give us a nice layout at ``resources/views/layouts/app.blade.php``.

Now create a model::

    $ vagga artisan make:model --migration Article

This will create a new model at ``app/Article.php`` and its respective migration
at ``database/migrations/2016_03_24_172211_create_articles_table.php`` (yours
will have a slightly different name).

Open the migration file and tell it to add two fields, ``title`` and ``body``,
to the database table for our Article model:

.. code-block:: php

    <?php

    use Illuminate\Database\Schema\Blueprint;
    use Illuminate\Database\Migrations\Migration;

    class CreateArticlesTable extends Migration
    {
        public function up()
        {
            Schema::create('articles', function (Blueprint $table) {
                $table->increments('id');
                $table->string('title', 100);
                $table->text('body');
                $table->timestamps();
            });
        }

        public function down()
        {
            Schema::drop('articles');
        }
    }

Open ``routes/web.php`` and setup routing:

.. code-block:: php

    <?php
    Route::get('/', 'ArticleController@index');
    Route::resource('/article', 'ArticleController');

    Auth::routes();

    Route::get('/home', 'HomeController@index');

Create our controller::

    $ vagga artisan make:controller --resource ArticleController

This will create a controller at ``app/Http/Controllers/ArticleController.php``
populated with some CRUD method stubs.

Now change the controller to actually do something:

.. code-block:: php

    <?php

    namespace App\Http\Controllers;

    use Illuminate\Http\Request;

    use App\Article;

    class ArticleController extends Controller
    {
        public function index()
        {
            $articles = Article::orderBy('created_at', 'asc')->get();
            return view('article.index', [
               'articles' => $articles
            ]);
        }

        public function create()
        {
            return view('article.create');
        }

        public function store(Request $request)
        {
            $this->validate($request, [
                'title' => 'required|max:100',
                'body' => 'required'
            ]);

            $article = new Article;
            $article->title = $request->title;
            $article->body = $request->body;
            $article->save();

            return redirect('/');
        }

        public function show(Article $article)
        {
            return view('article.show', [
                'article' => $article
            ]);
        }

        public function edit(Article $article)
        {
            return view('article.edit', [
                'article' => $article
            ]);
        }

        public function update(Request $request, Article $article)
        {
            $article->title = $request->title;
            $article->body = $request->body;
            $article->save();

            return redirect('/');
        }

        public function destroy(Article $article)
        {
            $article->delete();
            return redirect('/');
        }
    }

Create the views for our controller:

.. code-block:: html

    <!-- resources/views/article/show.blade.php -->
    @extends('layouts.app')

    @section('content')
    <div class="container">
        <div class="row">
            <div class="col-md-8 col-md-offset-2">
                <h2>{{ $article->title }}</h2>
                <p>{{ $article->body }}</p>
            </div>
        </div>
    </div>
    @endsection

.. code-block:: html

    <!-- resources/views/article/index.blade.php -->
    @extends('layouts.app')

    @section('content')
    <div class="container">
        <div class="row">
            <div class="col-md-8 col-md-offset-2">
                <h2>Article List</h2>
                <a href="{{ url('article/create') }}" class="btn">
                    <i class="fa fa-btn fa-plus"></i>New Article
                </a>
                @if (count($articles) > 0)
                <table class="table table-bordered table-striped">
                    <thead>
                        <th>id</th>
                        <th>title</a></th>
                        <th>actions</th>
                    </thead>
                    <tbody>
                        @foreach($articles as $article)
                        <tr>
                            <td>{{ $article->id }}</td>
                            <td>{{ $article->title }}</td>
                            <td>
                                <a href="{{ url('article/'.$article->id) }}" class="btn btn-success">
                                    <i class="fa fa-btn fa-eye"></i>View
                                </a>
                                <a href="{{ url('article/'.$article->id.'/edit') }}" class="btn btn-primary">
                                    <i class="fa fa-btn fa-pencil"></i>Edit
                                </a>
                                <form action="{{ url('article/'.$article->id) }}"
                                        method="post" style="display: inline-block">
                                    {!! csrf_field() !!}
                                    {!! method_field('DELETE') !!}
                                    <button type="submit" class="btn btn-danger"
                                            onclick="if (!window.confirm('Are you sure?')) { return false; }">
                                        <i class="fa fa-btn fa-trash"></i>Delete
                                    </button>
                                </form>
                            </td>
                        </tr>
                        @endforeach
                    </tbody>
                </table>
                @endif
            </div>
        </div>
    </div>
    @endsection

.. code-block:: html

    <!-- resources/views/article/create.blade.php -->
    @extends('layouts.app')

    @section('content')
    <div class="container">
        <div class="row">
            <div class="col-md-8 col-md-offset-2">
                <h2>Create Article</h2>
                @include('common.errors')
                <form action="{{ url('article') }}" method="post">
                    {!! csrf_field() !!}
                    <div class="form-group">
                        <label for="id-title">Title:</label>
                        <input id="id-title" class="form-control" type="text" name="title" />
                    </div>
                    <div class="form-group">
                        <label for="id-body">Title:</label>
                        <textarea id="id-body" class="form-control" name="body"></textarea>
                    </div>
                    <button type="submit" class="btn btn-primary">Save</button>
                </form>
            </div>
        </div>
    </div>
    @endsection

.. code-block:: html

    <!-- resources/views/article/edit.blade.php -->
    @extends('layouts.app')

    @section('content')
    <div class="container">
        <div class="row">
            <div class="col-md-8 col-md-offset-2">
                <h2>Edit Article</h2>
                @include('common.errors')
                <form action="{{ url('article/'.$article->id) }}" method="post">
                    {!! csrf_field() !!}
                    {!! method_field('PUT') !!}
                    <div class="form-group">
                        <label for="id-title">Title:</label>
                        <input id="id-title" class="form-control"
                               type="text" name="title" value="{{ $article->title }}" />
                    </div>
                    <div class="form-group">
                        <label for="id-body">Title:</label>
                        <textarea id="id-body" class="form-control" name="body">{{ $article->body }}</textarea>
                    </div>
                    <button type="submit" class="btn btn-primary">Save</button>
                </form>
            </div>
        </div>
    </div>
    @endsection

And the view for the common errors:

.. code-block:: html

    <!-- resources/views/common/errors.blade.php -->
    @if (count($errors) > 0)
    <div class="alert alert-danger">
        <ul>
            @foreach ($errors->all() as $error)
                <li>{{ $error }}</li>
            @endforeach
        </ul>
    </div>
    @endif

Create a seeder to prepopulate our database::

    $ vagga artisan make:seeder ArticleSeeder

This will create a seeder class at ``database/seeds/ArticleSeeder.php``. Open it
and change it as follows:

.. code-block:: php

    <?php

    use Illuminate\Database\Seeder;

    use App\Article;

    class ArticleSeeder extends Seeder
    {
        private $articles = [
            ['title' => 'Article 1', 'body' => 'Lorem ipsum dolor sit amet'],
            ['title' => 'Article 2', 'body' => 'Lorem ipsum dolor sit amet'],
            ['title' => 'Article 3', 'body' => 'Lorem ipsum dolor sit amet'],
            ['title' => 'Article 4', 'body' => 'Lorem ipsum dolor sit amet'],
            ['title' => 'Article 5', 'body' => 'Lorem ipsum dolor sit amet']
        ];

        public function run()
        {
            if (Article::all()->count() > 0) {
                return;
            }

            foreach ($this->articles as $article) {
                $new = new Article;
                $new->title = $article['title'];
                $new->body = $article['body'];
                $new->save();
            }
        }
    }

Change ``database/seeds/DatabaseSeeder.php`` to include ``ArticleSeeder``:

.. code-block:: php

    <?php
    use Illuminate\Database\Seeder;

    class DatabaseSeeder extends Seeder
    {
        public function run()
        {
            $this->call(ArticleSeeder::class);
        }
    }

Change the ``run`` command to execute the migrations and seed our database:

.. code-block:: yaml

  commands:
    run: !Supervise
      description: run the laravel development server
      children:
        app: !Command
          container: laravel
          environ: *db_config
          run: |
            # wait for database to be ready before starting
            dsn="mysql:host=$DB_HOST;port=$DB_PORT"
            while ! php -r "new PDO('$dsn', '$DB_USERNAME', '$DB_PASSWORD');" 2> /dev/null; do
              echo 'Waiting for database'
              sleep 2
            done

            php artisan cache:clear
            php artisan config:clear
            php artisan migrate
            php artisan db:seed
            php artisan serve
        db: !Command
          # ...
        adminer: !Command
          # ...

If you run our project, you will see the articles we defined in the seeder class.
Try adding some articles, then access adminer at ``localhost:8001`` to inspect
the database.

Setup Redis
===========

Laravel can make use of `redis <https://redis.io/>`_ to perform tasks like
queues and events. In our project, we will use it to cache data from the
database. First, let's create a command to call composer:

.. code-block:: yaml

    commands:
      # ...
      composer: !Command
        container: app
        description: run compose cli
        environ: ❶
          COMPOSER_HOME: /usr/local/lib/composer
          COMPOSER_VENDOR_DIR: /usr/local/lib/composer/vendor
          COMPOSER_CACHE_DIR: /tmp
          COMPOSER_ALLOW_SUPERUSER: 1
        volumes:
          /usr/local/lib/composer/vendor: !Tmpfs ❷
          /tmp: !CacheDir composer-cache ❸
        run: [/usr/local/bin/composer]

* ❶ -- setup composer home, vendor dir, cache dir and allow running as root
* ❷ -- mount directory as Tmpfs to make it writeable
* ❸ -- mount composer cache directory

This command setup the environment needed by composer to run properly and mount
the composer cache volume to avoid downloading cached packages. The directory
``/usr/local/lib/composer/vendor`` needs to be writeable (composer will will put
packages there) so we mount it as Tmpfs.

Now let's install ``predis/predis``::

    $ vagga composer require predis/predis

With ``predis`` installed, we can proceed to create a container for Redis:

.. code-block:: yaml

    containers:
      redis:
        setup:
        - !Alpine v3.5
        - !Install [redis]

Add some yaml anchors on the ``run`` command so we can avoid repetition:

.. code-block:: yaml

    commands:
      run: !Supervise
        description: run the laravel development server
        children:
          app: !Command
            container: app
            environ: *db_config
            run: &app_cmd | # ❶
                # ...
          db: &db_cmd !Command ❷
            # ...
          adminer: &adminer_cmd !Command ❸
            # ...

* ❶ -- set an anchor at the ``app`` child command
* ❷ -- set an anchor at the ``db`` child command
* ❸ -- set an anchor at the ``adminer`` child command

Create the command to run with caching:

.. code-block:: yaml

    commands:
      # ...
      run-cached: !Supervise
        description: Start the laravel development server alongside redis
        children:
          cache: !Command
            container: redis
            run: redis-server --daemonize no --port 6380 --loglevel verbose ❶
          app: !Command
            container: app
            environ:
              <<: *db_config
              CACHE_DRIVER: redis
              REDIS_HOST: 127.0.0.1
              REDIS_PORT: 6380
            run: *app_cmd
          db: *db_cmd
          adminer: *adminer_cmd

* ❶ -- run redis as verbose so we see can see the cache working

Now let's change our controller to use caching:

.. code-block:: php

    <?php

    namespace App\Http\Controllers;

    use Illuminate\Http\Request;

    use App\Article;

    use Cache;

    class ArticleController extends Controller
    {
        public function index()
        {
            $articles = Cache::rememberForever('article:all', function() {
                return Article::orderBy('created_at', 'asc')->get();
            });
            return view('article.index', [
               'articles' => $articles
            ]);
        }

        public function create()
        {
            return view('article.create');
        }

        public function store(Request $request)
        {
            $this->validate($request, [
                'title' => 'required|max:100',
                'body' => 'required'
            ]);

            $article = new Article;
            $article->title = $request->title;
            $article->body = $request->body;
            $article->save();

            Cache::forget('article:all');

            return redirect('/');
        }

        public function show($id)
        {
            $article = Cache::rememberForever('article:'.$id, function() use ($id) {
                return Article::find($id);
            });
            return view('article.show', [
                'article' => $article
            ]);
        }

        public function edit(Article article)
        {
            return view('article.edit', [
                'article' => $article
            ]);
        }

        public function update(Request $request, Article $article)
        {
            $article->title = $request->title;
            $article->body = $request->body;
            $article->save();

            Cache::forget('article:'.$article->id);
            Cache::forget('article:all');

            return redirect('/');
        }

        public function destroy(Article $article)
        {
            $article->delete();
            Cache::forget('article:'.$article->id);
            Cache::forget('article:all');
            return redirect('/');
        }
    }

Now run our project with caching::

    $ vagga run-cached

Keep an eye on the console to see Laravel talking to redis, you will see
something like::

    3:M 15 Mar 15:20:06.418 - DB 0: 5 keys (0 volatile) in 8 slots HT.

Deploying to a shared server
============================

It's still common to deploy a php application to a shared server running a LAMP
stack (Linux, Apache, MySQL and PHP), but our container in its current state
isn't compatible with that approach. To solve this, we will create a command to
export our project almost ready to be deployed.

Before going to the command part, we will need a new container for this task:

.. code-block:: yaml

    containers:
      # ...
      exporter:
        setup:
        - !Ubuntu xenial
        - !UbuntuUniverse
        - !Install [php-mbstring, php-dom]
        - !Depends composer.json ❶
        - !Depends composer.lock ❶
        - !EnsureDir /usr/local/src/
        - !Copy ❷
          source: /work
          path: /usr/local/src/work
        - !ComposerInstall ❸
        - !Env
          COMPOSER_VENDOR_DIR: /usr/local/src/work/vendor ❹
        - !Sh |
          cd /usr/local/src/work
          rm -f export.tar.gz
          composer install --no-dev --prefer-dist \ ❺
            --optimize-autoloader
        volumes:
          /usr/local/src/work: !Snapshot ❻

* ❶ -- rebuild the container if dependencies change.
* ❷ -- copy our project into a directory inside the container.
* ❸ -- require Composer to be available.
* ❹ -- install composer dependencies into the directory we just copied.
* ❺ -- call ``composer`` binary directly, because using ``!ComposerDependencies``
  would make vagga try to find ``composer.json`` before starting the build.
* ❻ -- create a volume so we can manipulate the files in the copied directory.

Now let's create the command to export our container:

.. code-block:: yaml

    commands:
      # ...
      export: !Command
        container: exporter
        description: export project into tarball
        run: |
            cd /usr/local/src/work
            rm -f .env
            rm -f database/database.sqlite
            php artisan cache:clear
            php artisan config:clear
            php artisan route:clear
            php artisan view:clear
            rm storage/framework/sessions/*
            rm -rf tests
            echo APP_ENV=production >> .env
            echo APP_KEY=random >> .env
            php artisan key:generate
            php artisan optimize
            php artisan route:cache
            php artisan config:cache
            php artisan vendor:publish
            tar -czf export.tar.gz .env *
            cp -f export.tar.gz /work/

.. note:: Take this command as a mere example, hence you are encouraged to
  change it in order to better suit your needs.

The shell in the ``export`` command will make some cleanup, remove tests (we
don't need them in production) and create a minimal .env file with an APP_KEY
generated. Then it will compress everything into a file called ``export.tar.gz``
and copy it to our project directory.

Since the ``export`` command is quite long, it is a good candidate to be moved
to a separate file, for example:

.. code-block:: yaml

    commands:
      # ...
      export: !Command
        container: exporter
        description: export project into tarball
        run: [/bin/sh, export.sh]
