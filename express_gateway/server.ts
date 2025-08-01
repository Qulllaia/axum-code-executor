import express, { Request, Response } from 'express';


const app = express();
const amqplib = require('amqplib/callback_api');

const queue = 'verification_email'


function sendRabbitMQmessage(message: String){
    amqplib.connect('amqp://guest:guest@localhost:5672', (err: any, conn: any) => {
    if (err) throw err;   
    
    conn.createChannel((err: any, ch1: any) => {
        try{
            if (err) throw err;
            ch1.assertQueue(queue, {
                durable: true,
            });
            const sent =  ch1.sendToQueue(queue, 
                Buffer.from(message),
                { persistent: true }
            );
            if (!sent) {
                console.error('Сообщение не было отправлено в очередь');
            }
            return sent;
        }catch(e) {   
            console.log(e)
        }
      });
    
    })
}

app.get('/verify', (req, res)=> {
    const verifyToken = req.query.verify_token;
    console.log(verifyToken);
    if(verifyToken){
        const message = JSON.stringify({ verify_token: verifyToken?.toString() });
        const ers = sendRabbitMQmessage(message)
        res.json({
            "result": ers
        })
    }else {
        res.json({
            "result": "matched1"
        })
    }
})


app.listen(5000, ()=>{
    console.log("server started")
})