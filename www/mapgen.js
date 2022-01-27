function get_image() {
	var xhr = new XMLHttpRequest();
	xhr.onreadystatechange = function(){
	    if (this.readyState == 4 && this.status == 200) {
			//this.response is what you're looking for
			console.log("response " + this.responseText);
			var img = document.getElementById('map');
			img.src = 'data:image/png;base64,' + this.responseText;
	    }
	}
	xhr.open('GET', 'cgi-bin/mapgen?w=48&h=48');
	xhr.setRequestHeader('Accept', 'text/plain');
	xhr.responseType = 'text';
	xhr.send();  
}
