function get_image(width, height, road_count, building_count) {
	var xhr = new XMLHttpRequest();
	xhr.onreadystatechange = function() {
		if (this.readyState == 4 && this.status == 200) {
			console.log("response " + this.responseText);
			var img = document.getElementById('map');
			img.src = 'data:image/png;base64,' + this.responseText;
		}
	}
	xhr.open('GET', 'cgi-bin/mapgen?-w=' + width + '&-h=' + height + '&-r=' + road_count + '&-b=' + building_count);
	xhr.setRequestHeader('Accept', 'text/plain');
	xhr.responseType = 'text';
	xhr.send();  
}
