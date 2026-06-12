import { useEffect, useState } from 'react';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import {
  BarChartOutlined,
  ExportOutlined,
  HomeOutlined,
  ImportOutlined,
  ShoppingOutlined,
} from '@ant-design/icons';
import type { MenuProps } from 'antd';
import { Layout, Menu } from 'antd';

const { Sider, Header, Content, Footer } = Layout;

type MenuItem = Required<MenuProps>['items'][number];

const menuItems: MenuItem[] = [
  {
    key: 'products',
    icon: <ShoppingOutlined />,
    label: '商品管理',
    children: [{ key: '/products', label: '商品列表' }],
  },
  {
    key: 'warehouses',
    icon: <HomeOutlined />,
    label: '仓库管理',
    children: [{ key: '/warehouses', label: '仓库列表' }],
  },
  {
    key: 'inbounds',
    icon: <ImportOutlined />,
    label: '入库管理',
    children: [{ key: '/inbounds', label: '入库单' }],
  },
  {
    key: 'outbounds',
    icon: <ExportOutlined />,
    label: '出库管理',
    children: [{ key: '/outbounds', label: '出库单' }],
  },
  {
    key: 'inventory',
    icon: <BarChartOutlined />,
    label: '库存查询',
    children: [{ key: '/inventory', label: '库存看板' }],
  },
];

/** Walk the menu tree to find the parent key for a given leaf path. */
function parentKeyFor(path: string): string | undefined {
  for (const item of menuItems) {
    if (item && 'children' in item && item.children) {
      for (const child of item.children) {
        if (child?.key === path) return item.key as string;
      }
    }
  }
  return undefined;
}

export default function AppLayout() {
  const [collapsed, setCollapsed] = useState(false);
  const [openKeys, setOpenKeys] = useState<string[]>(['products']);

  const navigate = useNavigate();
  const location = useLocation();

  /* Keep the parent menu open when the route changes. */
  useEffect(() => {
    const pk = parentKeyFor(location.pathname);
    if (pk && !openKeys.includes(pk)) {
      setOpenKeys((prev) => [...prev, pk]);
    }
  }, [location.pathname]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleMenuClick: MenuProps['onClick'] = (e) => {
    if (e.key.startsWith('/')) {
      navigate(e.key);
    }
  };

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider collapsible collapsed={collapsed} onCollapse={setCollapsed}>
        <div
          style={{
            height: 32,
            margin: 16,
            color: '#fff',
            textAlign: 'center',
            fontWeight: 'bold',
            fontSize: collapsed ? 14 : 16,
          }}
        >
          {collapsed ? 'WMS' : 'WMS'}
        </div>
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={[location.pathname]}
          openKeys={openKeys}
          onOpenChange={setOpenKeys}
          onClick={handleMenuClick}
          items={menuItems}
        />
      </Sider>
      <Layout>
        <Header
          style={{
            background: '#fff',
            padding: '0 24px',
            fontSize: 18,
            fontWeight: 500,
            display: 'flex',
            alignItems: 'center',
          }}
        >
          WMS 仓库管理系统
        </Header>
        <Content
          style={{
            margin: 24,
            padding: 24,
            background: '#fff',
            borderRadius: 8,
            minHeight: 280,
          }}
        >
          <Outlet />
        </Content>
        <Footer style={{ textAlign: 'center' }}>
          Copyright © 2026 WMS System
        </Footer>
      </Layout>
    </Layout>
  );
}
