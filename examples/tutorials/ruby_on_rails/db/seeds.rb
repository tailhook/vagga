# This file should contain all the record creation needed to seed the database with its default values.
# The data can then be loaded with the rails db:seed command (or created alongside the database with db:setup).
#
# Examples:
#
#   movies = Movie.create([{ name: 'Star Wars' }, { name: 'Lord of the Rings' }])
#   Character.create(name: 'Luke', movie: movies.first)

if Article.count == 0
  Article.create([
    { title: 'Article 1', body: 'Lorem ipsum dolor sit amet' },
    { title: 'Article 2', body: 'Lorem ipsum dolor sit amet' },
    { title: 'Article 3', body: 'Lorem ipsum dolor sit amet' }
  ])
end
