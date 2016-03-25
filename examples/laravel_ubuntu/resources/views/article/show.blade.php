@extends('layouts.app')

@section('content')
    <h2>{{ $article->title }}</h2>
    <p>{{ $article->body }}</p>
@endsection
