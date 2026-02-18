/**
 * 苍梧 TsangWu 前端交互逻辑
 */

const API_BASE = '';

/**
 * 页面加载完成后初始化
 */
document.addEventListener('DOMContentLoaded', function() {
    checkSystemStatus();
});

/**
 * 滚动到指定区块
 */
function scrollToSection(sectionId) {
    const element = document.getElementById(sectionId);
    if (element) {
        element.scrollIntoView({ behavior: 'smooth' });
    }
}

/**
 * 显示创作弹窗
 */
function showCreateModal() {
    document.getElementById('createModal').classList.add('active');
}

/**
 * 隐藏创作弹窗
 */
function hideCreateModal() {
    document.getElementById('createModal').classList.remove('active');
}

/**
 * 开始创作
 */
async function startCreation() {
    const type = document.getElementById('createType').value;
    const desc = document.getElementById('createDesc').value;
    const dynasty = document.getElementById('dynastySelect').value;

    if (!desc.trim()) {
        alert('请输入创作描述');
        return;
    }

    const resultDiv = document.createElement('div');
    resultDiv.innerHTML = '<p>正在生成中，请稍候...</p>';

    try {
        const response = await fetch(`${API_BASE}/api/v1/culture/identify`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ text: desc })
        });

        const result = await response.json();

        if (result.code === 0) {
            alert('创作请求已提交！文化元素识别成功。');
            hideCreateModal();
        } else {
            alert('创作失败：' + (result.message || '未知错误'));
        }
    } catch (error) {
        alert('请求失败：' + error.message);
    }
}

/**
 * 文化元素识别
 */
async function identifyCulture() {
    const input = document.getElementById('cultureInput').value;
    const resultDiv = document.getElementById('cultureResult');

    if (!input.trim()) {
        resultDiv.innerHTML = '<p class="placeholder">请输入文本内容</p>';
        return;
    }

    resultDiv.innerHTML = '<p>识别中...</p>';

    try {
        const response = await fetch(`${API_BASE}/api/v1/culture/identify`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ text: input })
        });

        const result = await response.json();

        if (result.code === 0 && result.data) {
            const data = result.data;
            let html = '<div class="culture-result">';

            if (data.dynasty) {
                html += `<p><strong>朝代：</strong>${data.dynasty}</p>`;
            }

            if (data.labels && data.labels.length > 0) {
                html += '<p><strong>文化标签：</strong></p><div class="tags">';
                data.labels.forEach(label => {
                    html += `<span class="tag">${label}</span>`;
                });
                html += '</div>';
            }

            if (data.styles && data.styles.length > 0) {
                html += '<p><strong>风格：</strong></p><div class="tags">';
                data.styles.forEach(style => {
                    html += `<span class="tag">${style}</span>`;
                });
                html += '</div>';
            }

            if (data.elements && data.elements.length > 0) {
                html += '<p><strong>文化元素：</strong></p><div class="tags">';
                data.elements.forEach(elem => {
                    html += `<span class="tag">${elem}</span>`;
                });
                html += '</div>';
            }

            if (data.narrative_type) {
                html += `<p><strong>叙事类型：</strong>${data.narrative_type}</p>`;
            }

            html += '</div>';
            resultDiv.innerHTML = html;
        } else {
            resultDiv.innerHTML = '<p class="error">识别失败：' + (result.message || '未知错误') + '</p>';
        }
    } catch (error) {
        resultDiv.innerHTML = '<p class="error">请求失败：' + error.message + '</p>';
    }
}

/**
 * 检查系统状态
 */
async function checkSystemStatus() {
    const apiStatus = document.getElementById('apiStatus');
    const dbStatus = document.getElementById('dbStatus');
    const cultureStatus = document.getElementById('cultureStatus');

    try {
        const response = await fetch(`${API_BASE}/healthz`);
        if (response.ok) {
            apiStatus.textContent = '正常运行';
            apiStatus.classList.remove('error');
            dbStatus.textContent = '已连接';
            dbStatus.classList.remove('error');
        } else {
            throw new Error('Health check failed');
        }
    } catch (error) {
        apiStatus.textContent = '离线';
        apiStatus.classList.add('error');
        dbStatus.textContent = '未知';
        dbStatus.classList.add('error');
    }

    try {
        const response = await fetch(`${API_BASE}/api/v1/culture/identify`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ text: 'test' })
        });
        if (response.ok) {
            cultureStatus.textContent = '就绪';
            cultureStatus.classList.remove('error');
        } else {
            throw new Error('Culture engine check failed');
        }
    } catch (error) {
        cultureStatus.textContent = '离线';
        cultureStatus.classList.add('error');
    }
}

/**
 * 点击弹窗外部关闭
 */
document.addEventListener('click', function(e) {
    const modal = document.getElementById('createModal');
    if (e.target === modal) {
        hideCreateModal();
    }
});
