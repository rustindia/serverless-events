  // Called when the page loads to properly initialize the like button
  function onLoad() {
    getEvents();
  }
  
  window.onload = onLoad;
  
  // === UTILITIES ===
  const prodEnv = true;
  const API_BASE_URL = prodEnv ? "https://serverless-events-api.fermyon.app" : "http://127.0.0.1:3000";
  
  function getEvents() {
  
      fetch(`${API_BASE_URL}/api/`).then((res) => {
        //console.log(res)
        res.json().then((data) => {
            //console.log(data)
            setRustIndiaEvents(data);
          });
      });  
  }
  
  function setRustIndiaEvents(rievents) {
    const container = document.getElementById('rievents');

    rievents.forEach(event => {
        // Create HTML elements
        const eventDiv = document.createElement('div');
        const namePara = document.createElement('div');
        const datePara = document.createElement('div');
        const registerPara = document.createElement('div');
        const communityPara = document.createElement('div');
        // Set the text content of the HTML elements
        namePara.className = "col py-2 border-bottom-secondary-subtle";
        namePara.textContent = event.name;

        communityPara.className = "col-3 py-2 border-bottom-secondary-subtle";
        communityPara.textContent = event.community;

        datePara.className = "col-4 py-2 px-2 border-bottom-secondary-subtle";
        datePara.textContent = event.date;

        registerPara.className = "col-2 py-2 border-bottom-secondary-subtle";
        registerPara.innerHTML = '<a href='+ event.url +' target="_blank">Register</a>';
        
        // Append the HTML elements to the container
        eventDiv.classList.add("row")
        eventDiv.appendChild(namePara);
        eventDiv.appendChild(communityPara);
        eventDiv.appendChild(datePara);
        eventDiv.appendChild(registerPara);
        container.appendChild(eventDiv);
      });

  }