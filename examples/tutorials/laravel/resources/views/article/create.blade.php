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
