from django.views import generic
from .models import Article


class ArticleList(generic.ListView):
    model = Article
    paginate_by = 10


class ArticleDetail(generic.DetailView):
    model = Article
