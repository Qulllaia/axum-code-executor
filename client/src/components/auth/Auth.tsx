import { Navigate, useNavigate } from "react-router-dom"
import { ParentForm } from "../forms/ParentForm"
import axios from "axios";
import { useState } from "react";
import './Auth.css'

export const Auth = () => {

    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [isWaitingEmail, setIsWaitingEmail] = useState(false);

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

        if(result?.status === 200) {
            navigate('/code', { replace: true });
        }
    }

    const regHandler = async () => {
        axios.interceptors.request.use(config => {
            config.withCredentials = true;
            return config;
        });
        const result = await axios.post(`http://127.0.1.1:5000/reg`, { 
            email,
            password
        })
        .catch((error) => {
            console.log(error);
        })
        setIsWaitingEmail(true);

        const verify_result = await axios.get(`http://127.0.1.1:5000/verify/ping?verify_token=${result?.data.verify_result}`)

        if(verify_result?.status === 200) {
            navigate('/code', { replace: true });
        }
    }

    return <div>
        {
            isWaitingEmail ?
            <ParentForm
                isDialog={false}
                isOpen={true}
                setIsOpen={()=>{}}
            >
                <p className="verify-text">Verify your account with mail</p>
                <div className="loader-platform">
                    <span className="loader" id='loader'></span>
                </div>
                <p className="sub-text">Check your email and click on verification link</p>
            </ParentForm>
        :
            <ParentForm
                isDialog={false}
                isOpen={true}
                setIsOpen={()=>{}}
            >
                <p>Authoriazation</p>
                <input placeholder="Login" onChange={(e)=> setEmail(e.target.value)} ></input>
                <input placeholder="Password" type="password" onChange={(e)=> setPassword(e.target.value)} ></input> 
                <br/>
                <button onClick={loginHandler}>Sign in</button>
                <button onClick={regHandler}>Sign up</button>
            </ParentForm>
        }
    </div>
}