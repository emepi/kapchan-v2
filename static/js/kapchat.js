const kapchatState = {
  socket: null,
  rooms: null,
  users: [],
  current_room: null,
};

const connect = () => {
    const { location } = window

    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws'
    const wsUri = `${proto}://${location.host}/ws`

    console.log('Connecting...')
    kapchatState.socket = new WebSocket(wsUri)

    kapchatState.socket.onopen = () => {
      console.log('Connected')
      kapchatState.socket.send(JSON.stringify({
        event: 2,
      }));

      kapchatState.socket.send(JSON.stringify({
        event: 3,
      }));
    }

    kapchatState.socket.onmessage = ev => {
      let message = JSON.parse(ev.data);

      switch (message.event) {
        case 6:
          displayTimeout(message.message);
          break;

        case 5:
          updateChatRooms(message.data);
          break;

        case 4:
          updateUsers(message.data);
          break;
        
        case 3:
          addMessage(message.username, message.message, message.room)
          break;

        case 2:
          deleteUser(message.username);
          break;

        case 1:
          addUser(message.username);
          break;

        default:
      }
    }

    kapchatState.socket.onclose = () => {
      console.log('Disconnected')
      socket = null
    }
}

const displayTimeout = (msg) => {
  let container = document.querySelector(".chat-messages");
  let error = document.createElement("div");
  error.classList.add("chat-error");
  error.textContent = msg;

  container.appendChild(error);
}

const addMessage = (username, message, roomm) => {
  let room = kapchatState.rooms.get(roomm);
  room.add({
    username: username,
    message: message,
  });

  renderRoom(roomm);
}

const renderRoom = (roomm) => {
    let room = kapchatState.rooms.get(roomm);
    let container = document.querySelector(".chat-messages");
    container.innerHTML = "";

    room.forall(message => {
      let chatTextCont = document.createElement("div");
      chatTextCont.classList.add("chat-text-cont");

      let textBlock = document.createElement("div");
      textBlock.classList.add("text-block");

      let userBlock = document.createElement("span");
      userBlock.classList.add("user-block");

      userBlock.textContent = message.username + ":";
      textBlock.textContent = message.message;

      chatTextCont.appendChild(userBlock);
      chatTextCont.appendChild(textBlock);
      container.appendChild(chatTextCont);
    })
}

const deleteUser = (user) => {
  let index = kapchatState.users.indexOf(user);

  if (index != -1) {
    kapchatState.users.splice(index, 1);
    updateUsers(kapchatState.users);
  }
}

const addUser = (user) => {
  kapchatState.users.push(user);
  updateUsers(kapchatState.users);
}

const updateUsers = (users) => {
  kapchatState.users = users;

  let container = document.querySelector(".chat-users");
  container.innerHTML = "";

  users
  .filter((value, index, array) => array.indexOf(value) === index)
  .forEach((user) => {
    let userBlock = document.createElement("div");
    userBlock.classList.add("user-block");
    userBlock.textContent = user;

    container.appendChild(userBlock);
  })
}

const updateChatRooms = (rooms) => {
  kapchatState.rooms = new Map();
  kapchatState.current_room = rooms[0];

  rooms.forEach((room) => {
    kapchatState.rooms.set(room, make_CRS_Buffer(50))
  });

  let container = document.querySelector(".chat-rooms");
  container.innerHTML = "";

  rooms.forEach((room) => {
    let roomBlock = document.createElement("div");
    roomBlock.classList.add("room-block");
    roomBlock.setAttribute("id", room);
    roomBlock.textContent = room;

    if (room == kapchatState.current_room) {
      roomBlock.classList.add("room-block--active");
    }

    roomBlock.addEventListener("click", e => {
      let previous_room = document.getElementById(kapchatState.current_room);
      previous_room.classList.remove("room-block--active");
      kapchatState.current_room = room;
      roomBlock.classList.add("room-block--active");
      renderRoom(room);
    })

    container.appendChild(roomBlock);
  })
}

const disconnect = () => {
  if (kapchatState.socket) {
    log('Disconnecting...')
    socket.close()
    socket = null
  }
}

const sendMessage = () => {
  let input = document.getElementById("chat-input");

  if (input.value) {
    kapchatState.socket.send(JSON.stringify({
      event: 1,
      message: input.value,
      room: kapchatState.current_room,
    }));
  }

  input.value = "";
}

const sendFromText = (event) => {
  let key = event.keyCode;

  if (key === 13) {
    event.preventDefault();
    sendMessage()
  }
}

connect()

function make_CRS_Buffer(size) {
  return {
    A:  [],
    Ai: 0,
    Asz:    size,
    add:    function(value) {
      this.A[ this.Ai ] = value;
      this.Ai = (this.Ai + 1) % this.Asz;
    },
    forall: function(callback) {
      var mAi = this.A.length < this.Asz ? 0 : this.Ai;
      for (var i = 0; i < this.A.length; i++) {
        callback(this.A[ (mAi + i) % this.Asz ]);
      }
    }
  };
}

document.addEventListener("DOMContentLoaded", (event) => {
  const scrollingElement = document.getElementById("chat-messages");
  const config = { childList: true };

  const callback = function (mutationsList, observer) {
    for (let mutation of mutationsList) {
      if (mutation.type === "childList") {
        scrollingElement.scrollTo(0, scrollingElement.scrollHeight);
      }
    }
  };

  const observer = new MutationObserver(callback);
  observer.observe(scrollingElement, config);
});