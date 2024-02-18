import { useState } from "react";
import * as S from "./index.styles";
import AppInput from "~components/input";
import AppButton from "~components/button";
import axios from "axios";
import { useValidation } from "@/utils/validation";
import { useNavigate } from "react-router-dom";
import { useSelector, useDispatch } from "react-redux";
import { setToken } from "~store/auth";

export default function Login() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const navigate = useNavigate();
  const dispatch = useDispatch();
  // const { get, data } = useApi();
  const form = useValidation({
    email: {
      $value: email,
      isEmpty: {
        $validator: (value: string) => {
          return !!value;
        },
        $message: "Email is required",
      },
      emailFormatIsCorrect: {
        $validator: (value: string) => {
          return !value.includes("@");
        },
        $message: "@ must be included in email",
      },
    },
    password: {
      $value: password,
      isEmpty: {
        $validator: (value: string) => {
          return !!value;
        },
        $message: "Password is required",
      },
    },
  });

  const login = async (e) => {
    e.preventDefault();
    if (form.$anyInvalids) {
      form.$touch();
      return;
    }

    await axios
      .post("/auth/login", {
        username: email,
        password: password,
      })
      .then((response) => {
        dispatch(setToken(response.data.Token));
        navigate("/minesweeper");
      });
  };

  return (
    <S.container>
      <S.inputContainer>
        {/* {validation} */}
        <form onSubmit={login}>
          <AppInput
            label="email"
            onChange={(e) => {
              setEmail(e.target.value);
              form.email.$touch();
            }}
            validation={form.email}
            placeholder="Enter your email..."
          />
          <AppInput
            label="password"
            onChange={(e) => {
              setPassword(e.target.value);
              form.password.$touch();
            }}
            validation={form.password}
            placeholder="Enter your password..."
            type="password"
          />
          <AppButton type="submit">login</AppButton>
          <S.register>
            Don't have an account? <a href="/register">Sign up</a>
          </S.register>
        </form>
      </S.inputContainer>
    </S.container>
  );
}
