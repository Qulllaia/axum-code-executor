import { Navigate, useNavigate } from "react-router-dom"
import { ParentForm } from "../forms/ParentForm"
import axios from "axios";
import { useState } from "react";

export const Auth = () => {

    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');

    const navigate = useNavigate();

    const loginHandler = async () => {
        axios.interceptors.request.use(config => {
            config.withCredentials = true;
            return config;
        });
        const result = await axios.post(`http://127.0.1.1:5000/login`, { 
            email,
            password
        })
        .catch((error) => {
            console.log(error);
        })
        // await fetch('http://127.0.1.1:5000/login', {
        //     method:'POST',
        //     credentials: 'include',
        //     headers: {
        //         'Accept': 'application/json',
        //         'Content-Type': 'application/json',
        //     },
        //     body: JSON.stringify({
        //         email,
        //         password
        //     })
        // })
        // .then(response => response.json())
        // .then(data => console.log(data));

        if(result?.status === 200) {
            navigate('/code', { replace: true });
        }
        console.log(result)
    }

    return <div>
        <ParentForm
            isDialog={false}
            isOpen={true}
            setIsOpen={()=>{}}
        >
            <p>Authoriazation</p>
            <input placeholder="Login" onChange={(e)=> setEmail(e.target.value)} ></input>
            <input placeholder="Password" type="password" onChange={(e)=> setPassword(e.target.value)} ></input> 
            <br/>
            <button onClick={()=>{
                navigate('/code', { replace: true });
            }}>Sign in</button>
            <button onClick={loginHandler}>Sign up</button>
        </ParentForm>
    </div>
}