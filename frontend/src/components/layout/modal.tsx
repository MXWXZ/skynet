import { StringIntl } from '@/utils';
import { ExclamationCircleTwoTone } from '@ant-design/icons';
import { Modal, ModalFuncProps } from 'antd';

interface confirmProps {
  title: string;
  content: string;
  intl: StringIntl;
}

const confirm = (props: confirmProps & ModalFuncProps) => {
  const { title, content, intl, ...rest } = props;
  Modal.confirm({
    title: intl.get(title),
    content: intl.get(content),
    icon: <ExclamationCircleTwoTone twoToneColor="#faad14" />,
    okType: 'danger',
    okText: props.intl.get('app.ok'),
    cancelText: props.intl.get('app.cancel'),
    onCancel() {},
    ...rest,
  });
};

export default confirm;
