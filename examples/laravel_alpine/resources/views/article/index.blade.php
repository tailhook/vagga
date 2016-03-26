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
