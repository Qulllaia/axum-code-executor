import { Navigate, useNavigate } from "react-router-dom"
import { ParentForm } from "../forms/ParentForm"

export const Auth = () => {
    const navigate = useNavigate();
    return <div>
        <ParentForm
            isDialog={false}
            isOpen={true}
            setIsOpen={()=>{}}
        >
            <p>Authoriazation</p>
            <input placeholder="Login"></input>
            <input placeholder="Password" type="password"></input> 
            <br/>
            <button onClick={()=>{
                navigate('/code', { replace: true });
            }}>Sign in</button>
            <button onClick={()=>{
                navigate('/code', { replace: true });
            }}>Sign up</button>
        </ParentForm>
    </div>
}