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
