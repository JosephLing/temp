json.(@data, :id, :title, :description)

if @data.owner
  json.(@data, :read_count)
end

json.editor do
  json.name @data.editor&.id
  json.id @data.editor&.id
end

json.pages(@data.pages)

json.uploads  @data.uploads do | upload |
  json.(upload, :id, :stored_filename, :user_filename, :file_type)
  if @options && @options[:include_upload_links]
    json.url upload.download_link
  end
end

# json.creator do
#   json.name @data.creator.safe_name(@user)
#   if @data.admin?
#     json.email @data.creator.email
#     if @data.creator.account
#       json.account do
#         json.name @c.creator.account.name
#       end
#     end
#   end
# end