function createTable( json_data ) {
  var table = document.querySelector( "#temp_table"  );
  json_data.forEach( function( entry ) {
    var tr = document.createElement( 'tr' );
    var td_caption = document.createElement( 'td' );
    var caption = document.createTextNode( entry[ "caption" ] );
    td_caption.appendChild( caption );
    td_caption.style.width = "400px";
    tr.appendChild( td_caption );
    var td_temp = document.createElement( 'td' );
    var temperature = document.createTextNode( entry[ "temperature" ] );
    td_temp.appendChild( temperature );
    td_temp.style.width = "100px";
    tr.appendChild( td_temp );
    table.appendChild( tr );
  });
}

var xhr = new XMLHttpRequest();
xhr.responseType = "json"
xhr.open('GET', '/get_temperatures', true);
xhr.onreadystatechange = function (){
  if (xhr.readyState != 4 ) {
    console.log( xhr.readyState );
    return;
  }

  if (xhr.status != 200) {
    alert( xhr.status + ': ' + xhr.statusText );
  } else {
    createTable( xhr.response );
  }
}
xhr.send();
