import Footer from '@/components/footer';
import ReCAPTCHA from '@/components/recaptcha';
import { getIntl, postAPI } from '@/utils';
import { LockOutlined, UserOutlined } from '@ant-design/icons';
import { LoginForm, ProFormCheckbox, ProFormText } from '@ant-design/pro-form';
import { Helmet, SelectLang, history, useModel } from '@umijs/max';
import { Form } from 'antd';
import _ from 'lodash';
import { useEffect } from 'react';
import { FormattedMessage } from 'react-intl';
import Recaptcha from 'react-recaptcha';
import styles from './index.less';

const handleSubmit = async (
  values: Record<string, any>,
  refresh: () => Promise<void>,
  recaptcha: Recaptcha | null,
) => {
  postAPI('/signin', values)
    .then((rsp) => {
      if (rsp && rsp.code === 0)
        refresh().then(() => {
          setTimeout(() => {
            window.location.href = '/dashboard'; // redirect to reload plugin list
          }, 1000);
        });
    })
    .finally(() => {
      recaptcha?.reset();
    });
};

const Login = () => {
  const intl = getIntl();
  const { setting, getSetting } = useModel('setting');
  const { initialState, refresh } = useModel('@@initialState');
  let recaptcha: Recaptcha | null;

  useEffect(() => {
    if (initialState?.signin) history.push('/dashboard');
    getSetting();
  }, []);

  return (
    <div className={styles.container}>
      <Helmet>
        <title>{intl.get('titles.index')}</title>
      </Helmet>
      <div className={styles.lang} data-lang>
        <SelectLang reload={false} />
      </div>
      <div className={styles.content}>
        <LoginForm
          title="Skynet"
          subTitle={intl.get('pages.index.subtitle')}
          initialValues={{
            autoLogin: true,
          }}
          onFinish={async (values) => {
            await handleSubmit(values, refresh, recaptcha);
          }}
        >
          <ProFormText
            name="username"
            fieldProps={{
              size: 'large',
              prefix: <UserOutlined />,
              maxLength: 32,
            }}
            placeholder={intl.get('pages.index.username')}
            rules={[
              {
                required: true,
                message: (
                  <FormattedMessage id="pages.index.username.required" />
                ),
              },
            ]}
          />
          <ProFormText.Password
            name="password"
            fieldProps={{
              size: 'large',
              prefix: <LockOutlined />,
            }}
            placeholder={intl.get('pages.index.password')}
            rules={[
              {
                required: true,
                message: (
                  <FormattedMessage id="pages.index.password.required" />
                ),
              },
            ]}
          />

          {!_.isEmpty(setting) && setting['recaptcha.enable'] === true && (
            <Form.Item
              name="g-recaptcha-response"
              rules={[
                {
                  required: true,
                  message: (
                    <FormattedMessage id="pages.index.captcha.required" />
                  ),
                },
              ]}
            >
              <ReCAPTCHA
                innerRef={(e) => (recaptcha = e)}
                url={setting['recaptcha.url']}
                sitekey={setting['recaptcha.sitekey']}
              />
            </Form.Item>
          )}

          <div style={{ marginBottom: 24 }}>
            <ProFormCheckbox noStyle name="remember">
              <FormattedMessage id="pages.index.rememberme" />
            </ProFormCheckbox>
          </div>
        </LoginForm>
      </div>
      <Footer />
    </div>
  );
};

export default Login;
