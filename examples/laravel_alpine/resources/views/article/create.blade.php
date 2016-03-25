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
