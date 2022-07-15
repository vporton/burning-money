import { useState } from "react";
import { backendUrlPrefix } from "./config";

// TODO: Make no need to reload page after login.

export function Login() {
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    async function do_login() {
        await window.fetch(backendUrlPrefix + "/login", {
            method: "POST",
            headers: {'content-type': 'application/x-www-form-urlencoded'},
            // mode: 'cors',
            body: new URLSearchParams({ email, password }),
        })
            .catch(e => alert(e.error));
        alert("Logged in. Please, reload the page.");
    }
    return (
        <>
            <p>Login: <input onChange={e => setEmail(e.target.value)}/></p>
            <p>Password: <input onChange={e => setPassword(e.target.value)} type="password"/></p>
            <p><button onClick={do_login}>Login</button></p>
        </>
    );
}

export function Register() {
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [firstName, setFirstName] = useState("");
    const [lastName, setLastName] = useState("");
    async function do_register() {
        await window.fetch(backendUrlPrefix + "/login", {
            method: "POST",
            headers: {'content-type': 'application/json'},
            body: JSON.stringify({email, password, first_name: firstName, last_name: lastName}),
        })
            .catch(e => alert(e));
        alert("Registered. Please, reload the page.");
    }
    return (
        <>
            <p>Login: <input onChange={e => setEmail(e.target.value)}/></p>
            <p>Password: <input onChange={e => setPassword(e.target.value)} type="password"/></p>
            <p>First name: <input onChange={e => setFirstName(e.target.value)}/></p>
            <p>Last name: <input onChange={e => setLastName(e.target.value)}/></p>
            <p><button onClick={do_register}>Register</button></p>
        </>
    );
}