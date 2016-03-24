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
