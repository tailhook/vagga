==========================
Building a Laravel project
==========================

This example will show how to create a simple Laravel project using vagga.

* `Creating the project structure`_
* `Adding some code`_
* `Caching with redis`_


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
2. Setup application environment
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

Setup application environment
-----------------------------

Laravel uses `dotenv`_ to load configuration into environment automatically from
a ``.env`` file, but we won't use that. Instead, we will tell vagga to set the
environment for us:

.. code-block:: yaml

    containers:
      laravel:
        environ: &env
          APP_ENV: development ❶
          APP_DEBUG: true ❷
          APP_KEY: YourRandomGeneratedEncryptionKey ❸
        setup:
        - !Alpine v3.3
        - !Env { <<: *env } ❹
        - !ComposerDependencies

* ❶ -- the "environment" our application will run (development, testing, production)
* ❷ -- enable debug mode
* ❸ -- a random, 32 character string used by encryption service
* ❹ -- inherit environment during build

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
          ENV_CONTAINER: 1 ❶
          APP_ENV: development
          APP_DEBUG: true
          APP_KEY: YourRandomGeneratedEncryptionKey
        setup:
        - !Alpine v3.3
        - !Env { <<: *env }
        - !ComposerDependencies

* ❶ -- tell our application we are running on a container

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
        run: |
            php artisan cache:clear ❶
            php artisan config:clear ❶
            php artisan serve

* ❶ -- clear application cache to prevent previous runs from intefering on
  subsequent runs.

Now run::

    $ vagga run

And visit ``localhost:8000``. If everithing was fine, you will see Laravel
default page saying "Laravel 5".

Adding some code
================

Now that we have our project working, let's add some code to it.

First, let's use ``artisan`` to scaffold authentication::

    $ vagga _run php artisan make:auth

This will give us a nice layout at ``resources/views/layouts/app.blade.php``.

Then, add a couple system dependencies needed for ``artisan`` and ``sqlite`` to
work properly with our project:

.. code-block:: yaml

    containers:
      laravel:
        environ: &env
          ENV_CONTAINER: 1
          APP_ENV: development
          APP_DEBUG: true
          APP_KEY: YourRandomGeneratedEncryptionKey
        setup:
        - !Alpine v3.3
        - !Env { <<: *env }
        - !Install
          - php-ctype ❶
          - php-pdo_sqlite ❷
        - !ComposerDependencies

* ❶ -- extension needed for ``artisan``
* ❷ -- PDO extension for sqlite.

Let's ensure we are sqlite as the default database. Open ``config/database.php``
and change the line ``'default' => env('DB_CONNECTION', 'mysql'),`` as follows:

.. code-block:: php

    <?php
    // ...
    'default' => env('DB_CONNECTION', 'sqlite'),

Now create a model::

    $ vagga _run laravel php artisan make:model --migration Article

This will create a new model at ``app/Article.php`` and its respective migration
at ``database/migrations/2016_03_24_172211_create_articles_table.php``. Since
migrations are timestamped, your migration will have a slightly different name.

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

Open ``app/routes.php`` and setup routing:

.. code-block:: php

    <?php
    Route::group(['middleware' => ['web']], function () {
        Route::auth();

        Route::get('/', 'ArticleController@index');
        Route::resource('/article', 'ArticleController');
        Route::get('/home', 'HomeController@index');
    });

Create our controller::

    $ vagga _run laravel php artisan make:controller --resource ArticleController

This will create a controller at ``app/ArticleController.php`` populated with
some CRUD method stubs.

Now change the controller to actually do something:

.. code-block:: php

    <?php
    namespace App\Http\Controllers;

    use Illuminate\Http\Request;

    use App\Http\Requests;
    use App\Http\Controllers\Controller;
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

And finally create the views for our controller:

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

.. code-block:: html

    <!-- resources/views/common/error.blade.php -->
    @if (count($errors) > 0)
    <div class="alert alert-danger">
        <ul>
            @foreach ($errors->all() as $error)
                <li>{{ $error }}</li>
            @endforeach
        </ul>
    </div>
    @endif

Caching with redis
==================

Many projects use some caching strategy to speed things up. Let's try caching
using `redis <http://redis.io>`_.

Add ``predis/predis``, a pure php redis client, to our ``composer.json``:

.. code-block:: json

    "require": {
        "php": ">=5.5.9",
        "laravel/framework": "5.2.*",
        "predis/predis": "~1.0"
    },

By default, Composer will pick dependencies from ``composer.lock`` and just
display a warning about the out of date lock file, meaning it won't install the
redis client package. To solve that, simply remove the lock file::

    $ rm composer.lock

.. note:: We could have put an option in vagga to use ``composer update``
  instead of ``composer install``, but we, as developers, are likely to forget
  such an option active and it would end up with anyone working on the project
  having different versions of its dependencies. Besides, you can always add a
  build step to call ``composer update`` manually.

Create a container for ``redis``:

.. code-block:: yaml

    containers:
      # ...
      redis:
        setup:
        - !Alpine v3.3
        - !Install [redis]

Create the command to run with caching:

.. code-block:: yaml

    commands:
      # ...
      run-cached: !Supervise
        description: Start the laravel development server alongside memcached
        children:
          cache: !Command
            container: redis
            run: redis-server --dir /tmp --dbfilename redis.rdb ❶
          app: !Command
            container: laravel
            environ: ❷
              CACHE_DRIVER: redis
              REDIS_HOST: 127.0.0.1
              REDIS_PORT: 6379
            run: |
                php artisan cache:clear
                php artisan config:clear
                php artisan serve

* ❶ -- set the redis db file to a temporary directory
* ❷ -- set the environment for using redis

Now let's change our controller to use caching:

.. code-block:: php

    <?php

    namespace App\Http\Controllers;

    use Illuminate\Http\Request;

    use App\Http\Requests;
    use App\Http\Controllers\Controller;
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

        public function edit($id)
        {
            $article = Cache::rememberForever('article:'.$id, function() use ($id) {
                return Article::find($id);
            });
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

To see Laravel talking to redis, open another console tab and run::

    $ vagga _run redis redis-cli monitor

You can now add and remove some articles to see the redis log on the console.

Let's try Postgres
==================

When deploying to production, you will certainly use a database server, so let's
try Postgres.

First, add the system dependency ``php-pdo_pgsql`` to our container:

.. code-block:: yaml

    containers:
      laravel:
        environ: &env
          ENV_CONTAINER: 1
          APP_ENV: development
          APP_DEBUG: true
          APP_KEY: YourRandomGeneratedEncryptionKey
        setup:
        - !Alpine v3.3
        - !Install
          - php-ctype
          - php-pdo_sqlite
          - php-pdo_pgsql
        - !Env { <<: *env }
        - !ComposerDependencies

Create a container for our database:

.. code-block:: yaml

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

Then add a command to run our project with Postgres:

.. code-block:: yaml

    run-postgres: !Supervise
      description: Start the laravel development server using Postgres database
      children:
        app: !Command
          container: laravel
          environ:
            DB_CONNECTION: pgsql
            DB_HOST: 127.0.0.1
            DB_PORT: 5433
            DB_DATABASE: test
            DB_USERNAME: vagga
            DB_PASSWORD: vagga
          run: |
              touch /work/.dbcreation # Create lock file
              while [ -f /work/.dbcreation ]; do sleep 0.2; done # Acquire lock
              php artisan cache:clear
              php artisan config:clear
              php artisan migrate
              php artisan db:seed
              php artisan serve
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

Now lets create a seeder to populate our database everytime we run our project::

    $ vagga _run laravel php artisan make:seeder ArticleSeeder

This will create our seeder class at ``database/seeds/ArticleSeeder.php``. Open
it and change it as follows:

.. code-block:: php

    <?php
    use Illuminate\Database\Seeder;
    use App\Article;

    class ArticleSeeder extends Seeder
    {
        public function run()
        {
            $article = [
                ['title' => 'Article 1', 'body' => 'Lorem ipsum dolor sit amet'],
                ['title' => 'Article 2', 'body' => 'Lorem ipsum dolor sit amet'],
                ['title' => 'Article 3', 'body' => 'Lorem ipsum dolor sit amet'],
                ['title' => 'Article 4', 'body' => 'Lorem ipsum dolor sit amet'],
                ['title' => 'Article 5', 'body' => 'Lorem ipsum dolor sit amet']
            ];
            foreach ($articles as $article) {
                $new = new Article;
                $new->title = $article['title'];
                $new->body = $article['body'];
                $new->save();
            }
        }
    }

Change ``database/seeds/DatabaseSeeder.php`` to include our ArticleSeeder:

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


Now run our project::

    $ vagga run-postgres

Deploying to a shared server
============================

It's still common to deploy a php application to a shared server running a LAMP
stack (Linux, Apache, MySql and PHP), but our container in its current state
isn't compatible with that approach. To solve this, we will create a command to
export our project almost ready to be deployed.

Before going to the command part, we will need a new container for this task:

.. code-block:: yaml

    containers:
      # ...
      exporter:
        setup:
        - !Alpine v3.3
        - !EnsureDir /usr/local/src/
        - !Copy
          source: /work
          path: /usr/local/src/work
        - !ComposerInstall
        - !Env
          COMPOSER_VENDOR_DIR: /usr/local/src/work/vendor
        - !Sh |
            cd /usr/local/src/work
            rm -f export.tar.gz
            composer install \
            --no-dev --prefer-dist --optimize-autoloader
        volumes:
          /usr/local/src/work: !Snapshot

There is a lot going on in this container, but let me explain it:

We start by copying our project into a directory inside the container:

.. code-block:: yaml

    - !EnsureDir /usr/local/src/
    - !Copy
      source: /work
      path: /usr/local/src/work

Then we require composer to be available:

.. code-block:: yaml

    - !ComposerInstall

Set the environment to install dependencies in the directory we just copied our
project into:

.. code-block:: yaml

    - !Env
      COMPOSER_VENDOR_DIR: /usr/local/src/work/vendor

And finnaly we ``cd`` to the referred directory, remove any ``export.tar.gz``
(our export file) it may contain and run ``composer install`` with some flags to
optimize dependency installation and autoloader:

.. code-block:: yaml

    - !Sh |
        cd /usr/local/src/work
        rm -f export.tar.gz
        composer install \
        --no-dev --prefer-dist --optimize-autoloader

We also create a volume so we can freely manipulate the files in that directory:

.. code-block:: yaml

    volumes:
      /usr/local/src/work: !Snapshot

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
            php artisan optimize
            php artisan route:cache
            php artisan vendor:publish
            echo APP_ENV=production >> .env
            echo APP_KEY=random >> .env
            php artisan key:generate
            tar -czf export.tar.gz .env *
            cp -f export.tar.gz /work/

.. note:: Take this command as a mere example, hence you are encouraged to
  change it in order to better suit your needs.

The shell in the ``export`` command will make some cleanup, remove tests (we
don't need them in production) and create a minimal .env file with an APP_KEY
generated. Then it will compress everything into a file called ``export.tar.gz``
and copy it to our project directory.
