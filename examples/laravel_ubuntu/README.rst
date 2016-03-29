==========================
Building a Laravel project
==========================

This example will show how to create a simple Laravel project using vagga.

* `Creating the project structure`_
* `Setup the database`_
* `Adding some code`_
* `Trying out memcached`_
* `Deploying to a shared server`_


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
        - !Ubuntu trusty
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
        - !Ubuntu trusty
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
        - !Ubuntu trusty
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
        - !Ubuntu trusty
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
    if (getenv('ENV_CONTAINER')) {
        require '/usr/local/lib/composer/vendor/autoload.php';
    } else {
        require __DIR__.'/../vendor/autoload.php';
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

Setup the database
==================

Every PHP project needs a database, and ours is not different, so let's create a
container for our database:

.. code-block:: yaml

    containers:
      # ...
      mysql:
        setup:
        - !Alpine v3.3
        - !Install
          - mariadb ❶
          - mariadb-client
          - php-cli ❷
        - !EnsureDir /data
        - !EnsureDir /opt/adminer
        - !Download ❷
          url: https://www.adminer.org/static/download/4.2.4/adminer-4.2.4-mysql.php
          path: /opt/adminer/adminer.php
        - !Download ❸
          url: https://raw.githubusercontent.com/vrana/adminer/master/designs/nette/adminer.css
          path: /opt/adminer/adminer.css
        environ: &db_config ❹
          DB_DATABASE: vagga
          DB_USERNAME: vagga
          DB_PASSWORD: vagga
          DB_HOST: 127.0.0.1
          DB_PORT: 3307
          DB_DATA_DIR: /data
        volumes:
          /data: !Tmpfs
            size: 200M
            mode: 0o700

* ❶ -- `mariadb`_ is a drop in replacement for mysql.
* ❷ -- we need php to run `adminer`_, a small database administration tool.
* ❸ -- a better style for adminer.
* ❹ -- set an yaml anchor so we can reference it in our run command.

Now change our ``run`` command to start the database alongside our project:

.. code-block:: yaml

    commands:
      run: !Supervise
        description: run the laravel development server
        children:
          app: !Command
            container: laravel
            environ: *db_config
            run: |
                touch /work/.dbcreation # Create lock file
                while [ -f /work/.dbcreation ]; do sleep 0.2; done # Acquire lock
                php artisan cache:clear
                php artisan config:clear
                php artisan serve
          db: !Command
            container: mysql
            run: |
                mysql_install_db --datadir=$DB_DATA_DIR
                mkdir /run/mysqld
                mysqld_safe --user=root --datadir=$DB_DATA_DIR \
                  --bind-address=$DB_HOST --port=$DB_PORT \
                  --no-auto-restart --no-watch
                while [ ! -S /run/mysqld/mysqld.sock ]; do sleep 0.2; done # wait for server to be ready
                mysqladmin create $DB_DATABASE
                mysql -e "CREATE USER '$DB_USERNAME'@'localhost' IDENTIFIED BY '$DB_PASSWORD';"
                mysql -e "GRANT ALL PRIVILEGES ON $DB_DATABASE.* TO '$DB_USERNAME'@'localhost';"
                mysql -e "FLUSH PRIVILEGES;"
                rm /work/.dbcreation # Release lock
                php -S 127.0.0.1:8800 -t /opt/adminer # run adminer

.. _`mariadb`: http://mariadb.org/
.. _`adminer`: https://www.adminer.org

And run our project::

    $ vagga run

To access adminer, visit ``localhost:8800``, fill in the ``server`` field with
``127.0.0.1:3307`` and the other fields with "vagga" (the username and password
we defined).

Adding some code
================

Now that we have our project working and our database is ready, let's add some.

First, let's use ``artisan`` to scaffold authentication::

    $ vagga _run laravel php artisan make:auth

This will give us a nice layout at ``resources/views/layouts/app.blade.php``.

Then, add a the php mysql module to our container

.. code-block:: yaml

    containers:
      laravel:
        environ: &env
          ENV_CONTAINER: 1
          APP_ENV: development
          APP_DEBUG: true
          APP_KEY: YourRandomGeneratedEncryptionKey
        setup:
        - !Ubuntu trusty
        - !Env { <<: *env }
        - !Install
          - php5-mysql
        - !ComposerDependencies

Now create a model::

    $ vagga _run laravel php artisan make:model --migration Article

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

Create a seeder to prepopulate our database::

    $ vagga _run laravel php artisan make:seeder ArticleSeeder

This will create a seeder class at ``database/seeds/ArticleSeeder.php``. Open it
and change it as follows:

.. code-block:: php

    <?php
    use Illuminate\Database\Seeder;
    use App\Article;

    class ArticleSeeder extends Seeder
    {
        public function run()
        {
            $articles = [
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
              touch /work/.dbcreation # Create lock file
              while [ -f /work/.dbcreation ]; do sleep 0.2; done # Acquire lock
              php artisan cache:clear
              php artisan config:clear
              php artisan migrate
              php artisan db:seed
              php artisan serve
        db: !Command
          # ...

If you run our project, you will see the articles we defined in the seeder class.
Try adding some articles, then access adminer at ``localhost:8800`` to inspect
the database.

Trying out memcached
====================

Many projects use `memcached <http://memcached.org/>`_ to speed up things, so
let's try it out.

Activate Universe repository and add ``php5-memcached``, to our container:

.. code-block:: yaml

    containers:
      laravel:
        environ: &env
          ENV_CONTAINER: 1
          APP_ENV: development
          APP_DEBUG: true
          APP_KEY: YourRandomGeneratedEncryptionKey
        setup:
        - !Ubuntu trusty
        - !UbuntuUniverse
        - !Env { <<: *env }
        - !Install
          - php5-mysql
          - php5-memcached
        - !ComposerDependencies

Create a container for ``memcached``:

.. code-block:: yaml

    containers:
      # ...
      memcached:
        setup:
        - !Alpine v3.3
        - !Install [memcached]

Add some yaml anchors on the ``run`` command so we can avoid repetition:

.. code-block:: yaml

    commands:
      run: !Supervise
        description: run the laravel development server
        children:
          app: !Command
            container: laravel
            environ: *db_config
            run: &run_app | ❶
                # ...
          db: !Command
            container: mysql
            run: &run_db | ❷
                # ...

* ❶ -- set an anchor at the ``app`` child command
* ❷ -- set an anchor at the ``db`` child command

Create the command to run with caching:

.. code-block:: yaml

    commands:
      # ...
      run-cached: !Supervise
        description: Start the laravel development server alongside memcached
        children:
          cache: !Command
            container: memcached
            run: memcached -u memcached -vv ❶
          app: !Command
            container: laravel
            environ:
              <<: *db_config
              CACHE_DRIVER: memcached
              MEMCACHED_HOST: 127.0.0.1
              MEMCACHED_PORT: 11211
            run: *run_app
          db: !Command
            container: mysql
            run: *run_db

* ❶ -- run memcached as verbose so we see can see the cache working

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

Keep an eye on the console to see Laravel talking to memcached.

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
        - !Ubuntu trusty
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
            composer install \ ❺
              --no-dev --prefer-dist --optimize-autoloader
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
        run: [sh, export.sh]
