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


    /**
     * Run the database seeds.
     *
     * @return void
     */
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
