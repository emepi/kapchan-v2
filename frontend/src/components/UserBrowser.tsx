import { For, createResource, createSignal } from 'solid-js';
import { apiFetch } from '../scripts/connection';
import './UserBrowser.css';


interface UserData {
  id: number,
  access_level: number,
  username: string,
  email: string | null,
  created_at: string,
};

export function UserBrowser() {

  const fetchUsers = async (page: number): Promise<[UserData]> => {
    const offset = page * 50;
    const limit = (page + 1) * 50;

    return await apiFetch("/users?offset=" + offset + "&limit=" + limit, {
      method: "GET",
    })
    .then((res) => res.json());
  };

  const [page, _setPage] = createSignal(0);
  const [users] = createResource(page, fetchUsers);

  return (
    <table>
      <tbody>
        <tr>
          <th class="user-brws-tab">ID</th>
          <th class="user-brws-tab">Username</th>
          <th class="user-brws-tab">Email</th>
          <th class="user-brws-tab">Access Level</th>
          <th class="user-brws-tab">Created At</th>
        </tr>
        <For each={users()}>{(user: UserData) =>
          <tr>
            <td>{user.id}</td>
            <td>{user.username}</td>
            <td>{user.email}</td>
            <td>{user.access_level}</td>
            <td>{user.created_at}</td>
          </tr>
        }</For>
      </tbody>
    </table>
  );
}