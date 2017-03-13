<?php

use Illuminate\Database\Seeder;

// use ArticleSeeder;

class DatabaseSeeder extends Seeder
{
    /**
     * Run the database seeds.
     *
     * @return void
     */
    public function run()
    {
        $this->call(ArticleSeeder::class);
    }
}
