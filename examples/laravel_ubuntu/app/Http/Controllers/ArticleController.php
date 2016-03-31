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
