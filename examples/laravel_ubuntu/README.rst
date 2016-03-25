==========================
Building a Laravel project
==========================

This example will show how to create a simple Laravel project using vagga.

* `Creating the project structure`_
* `Adding some code`_
* `Caching with memcached`_


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
        run: php artisan serve

Now run::

    $ vagga run

And visit ``localhost:8000``. If everithing was fine, you will see Laravel
default page saying "Laravel 5".

Adding some code
================

Now that we have our project working, let's add some code to it.

First, let's add ``php5-sqlite`` to our container:

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
        - !UbuntuUniverse ❶
        - !Env { <<: *env }
        - !Install
          - php5-sqlite
        - !ComposerDependencies

* ❶ -- package ``php5-sqlite`` is provided by Ubuntu Universe.

Then, let's ensure we are sqlite as the default database. Open ``config/database.php``
and change the line ``'default' => env('DB_CONNECTION', 'mysql'),`` as follows:

.. code-block:: php

    <?php
    // ...
    'default' => env('DB_CONNECTION', 'sqlite'),

Now let's create a model::

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
        Route::get('/', 'ArticleController@index');
        Route::resource('article', 'ArticleController');
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

Create a layout:

.. code-block:: html

    <!-- resources/views/layouts/app.blade.php -->
    <!DOCTYPE html>
    <html>
    <head>
        <title>Vagga tutorial</title>
        <style>
            body {
                font-family: sans-serif;
            }
        </style>
    </head>
    <body>
        @yield('content')
    </body>
    </html>

And finally create the views for our controller:

.. code-block:: html

    <!-- resources/views/article/show.blade.php -->
    @extends('layouts.app')

    @section('content')
        <h2>{{ $article->title }}</h2>
        <p>{{ $article->body }}</p>
    @endsection

.. code-block:: html

    <!-- resources/views/article/index.blade.php -->
    @extends('layouts.app')

    @section('content')
        <h2>Article List</h2>
        <a href="{{ url('article/create') }}">New Article</a>
        @if (count($articles) > 0)
        <table>
            <thead>
                <th>id</th>
                <th>title</a></th>
                <th>actions</th>
            </thead>
            <tbody>
                @foreach($articles as $article)
                <tr>
                    <td>{{ $article->id }}</td>
                    <td>
                        <a href="{{ url('article/'.$article->id) }}">{{ $article->title }}</a>
                    </td>
                    <td>
                        <form action="{{ url('article/'.$article->id) }}" method="post">
                            {!! csrf_field() !!}
                            {!! method_field('DELETE') !!}
                            <button type="submit">Delete</button>
                        </form>
                    </td>
                </tr>
                @endforeach
            </tbody>
        </table>
        @endif
    @endsection

.. code-block:: html

    <!-- resources/views/article/create.blade.php -->
    @extends('layouts.app')

    @section('content')
        <h2>Create Article</h2>
        @include('common.errors')
        <form action="{{ url('article') }}" method="post">
            {!! csrf_field() !!}
            <label for="id-title">Title:</label>
            <input id="id-title" type="text" name="title" />
            <br />
            <label for="id-body">Title:</label>
            <textarea id="id-body" name="body"></textarea>
            <br />
            <button type="submit">Save</button>
        </form>
    @endsection

.. code-block:: html

    <!-- resources/views/article/edit.blade.php -->
    @extends('layouts.app')

    @section('content')
        <h2>Edit Article</h2>
        @include('common.errors')
        <form action="{{ url('article/'.$article->id) }}" method="post">
            {!! csrf_field() !!}
            {!! method_field('PUT') !!}
            <label for="id-title">Title:</label>
            <input id="id-title" type="text" name="title" value="{{ $article->title }}" />
            <br />
            <label for="id-body">Title:</label>
            <textarea id="id-body" name="body">{{ $article->body }}</textarea>
            <br />
            <button type="submit">Save</button>
        </form>
    @endsection

.. code-block:: html

    <!-- resources/views/common/error.blade.php -->
    @if (count($errors) > 0)
        <ul>
            @foreach ($errors->all() as $error)
                <li>{{ $error }}</li>
            @endforeach
        </ul>
    @endif

Caching with memcached
======================

Many projects use `memcached <http://memcached.org/>`_ to speed up things, so
let's try it out.

Add ``php-memcached`` to our container:

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
          - php5-sqlite
          - php5-memcached ❶
        - !ComposerDependencies

* ❶ -- memcached php extension

Create a container for ``memcached``:

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
        description: Start the laravel development server alongside memcached
        children:
          cache: !Command
            container: memcached
            run: memcached -u memcached -vv ❶
          app: !Command
            container: laravel
            environ: ❷
              CACHE_DRIVER: memcached
              MEMCACHED_HOST: 127.0.0.1
              MEMCACHED_PORT: 11211
            run: php artisan serve

* ❶ -- run memcached as verbose so we see can see the cache working
* ❷ -- set the environment for using memcached

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

Try to add and remove some articles and see Laravel talking to memcached on the
console.
