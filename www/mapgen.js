function get_image(preset) {
	document.getElementById('map').src = '';
	document.getElementById("status").textContent="Please wait..."
	var xhr = new XMLHttpRequest();
	xhr.onreadystatechange = function() {
		if (this.readyState == 4 && this.status == 200) {
			console.log("response " + this.responseText);
			document.getElementById('map').src = 'data:image/png;base64,' + this.responseText;
			document.getElementById("status").textContent="Done!"
		}
	}
	xhr.open('GET', '/mapgen-cgi/mapgen?-p=' + preset);
	xhr.setRequestHeader('Accept', 'text/plain');
	xhr.responseType = 'text';
	xhr.send();  
}
